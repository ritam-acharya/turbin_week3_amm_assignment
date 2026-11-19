use crate::error::ErrorCode;
use anchor_lang::prelude::*;

pub struct Response {
    pub x: u64,
    pub y: u64
}

pub struct SwapResponse {
    pub deposit: u64,
    pub fee: u64,
    pub withdraw: u64
}


pub struct ConstantProduct {
    pub x: u64, // Balance of Token X
    pub y: u64, // Balance of Token Y
    pub l: u64, // LP Token Balance
    pub fee: u64, // Fee in basis points, ie: 100 = 1%
    pub precision: u32
}

pub enum LiquidityPair {
    X,
    Y
}

impl ConstantProduct {
    pub fn init(x: u64, y: u64, l: u64, fee: u64, precision: Option<u32>) -> Result<ConstantProduct> {
        // Assert non-zero values of X and Y
        require!(x > 0, ErrorCode::InvalidAmount);
        require!(y > 0, ErrorCode::InvalidAmount);

        let precision = match precision {
            Some(p) => 10u32.checked_pow(p as u32).ok_or(ErrorCode::InvalidPrecision)?,
            None => {
                1_000_000
            }
        };
        
        // If L is zero, make it the higher value of either X or Y, as this will have less rounding errors
        let l = match l>0 {
            true => l,
            false => x.max(y)
        };

        Ok(ConstantProduct {
            x,
            y,
            l,
            fee,
            precision
        })
    }

    pub fn xy_deposit_amounts_from_l(x: u64, y: u64, l: u64, a: u64, precision: u32) -> Result<Response> {
        let ratio = (l as u128)
            .checked_add(a as u128).ok_or(ErrorCode::Overflow)?
            .checked_mul(precision as u128).ok_or(ErrorCode::Overflow)?
            .checked_div(l as u128).ok_or(ErrorCode::Overflow)?;
        let deposit_x = (x as u128)
            .checked_mul(ratio).ok_or(ErrorCode::Overflow)?
            .checked_div(precision as u128).ok_or(ErrorCode::Overflow)?
            .checked_sub(x as u128).ok_or(ErrorCode::Overflow)? as u64;
        let deposit_y = (y as u128)
            .checked_mul(ratio).ok_or(ErrorCode::Overflow)?
            .checked_div(precision as u128).ok_or(ErrorCode::Overflow)?
            .checked_sub(y as u128).ok_or(ErrorCode::Overflow)? as u64;
        Ok(Response {
            x: deposit_x,
            y: deposit_y
        })
    }

    pub fn xy_withdraw_amounts_from_l(x: u64, y: u64, l: u64, a: u64, precision: u32) -> Result<Response> {
        let ratio = ((l - a) as u128)
        .checked_mul(precision as u128).ok_or(ErrorCode::Overflow)?
        .checked_div(l as u128).ok_or(ErrorCode::Overflow)?;

        let withdraw_x = (x as u128)
            .checked_sub((x as u128)
                .checked_mul(ratio).ok_or(ErrorCode::Overflow)?
                .checked_div(precision as u128).ok_or(ErrorCode::Overflow)?
            ).ok_or(ErrorCode::Overflow)? as u64;

        let withdraw_y = (y as u128)
            .checked_sub((y as u128)
                .checked_mul(ratio).ok_or(ErrorCode::Overflow)?
                .checked_div(precision as u128).ok_or(ErrorCode::Overflow)?
            ).ok_or(ErrorCode::Overflow)? as u64;

        Ok(Response {
            x: withdraw_x, 
            y: withdraw_y
        })
    }

    pub fn x2_from_y_swap_amount(x: u64, y: u64, a: u64) -> Result<u64> {
        let k = Self::k_from_xy(x, y)?;
        let x_new = (y as u128).checked_add(a as u128).ok_or(ErrorCode::Overflow)?;
        Ok(k.checked_div(x_new).ok_or(ErrorCode::Overflow)? as u64)
    }

    pub fn k_from_xy(x: u64, y: u64) -> Result<u128> {
        require!(x > 0 && y > 0, ErrorCode::InvalidAmount);
        Ok((x as u128).checked_mul(y as u128).unwrap())
    }

    pub fn delta_x_from_y_swap_amount(x: u64, y: u64, a: u64) -> Result<u64> {
        Ok(x.checked_sub(Self::x2_from_y_swap_amount(x,y,a)?).ok_or(ErrorCode::Overflow)?)
    }

    pub fn y2_from_x_swap_amount(x: u64, y: u64, a: u64) -> Result<u64> {
        Self::x2_from_y_swap_amount(y,x,a)
    }

    pub fn delta_y_from_x_swap_amount(x: u64, y: u64, a: u64) -> Result<u64> {
        Self::delta_x_from_y_swap_amount(y,x,a)
    }

    pub fn swap_slippage(withdraw: u64, min: u64) -> Result<()> {
        require!(withdraw >= min, ErrorCode::SlippageExceeded);
        Ok(())
    }

    pub fn swap(&mut self, p: LiquidityPair, a: u64, min: u64) -> Result<SwapResponse> {
        let a2 = (a as u128)
            .checked_mul((10_000 - self.fee) as u128)
            .ok_or(ErrorCode::Overflow)?
            .checked_div(10_000)
            .ok_or(ErrorCode::Overflow)? as u64;
    
        let (new_x, new_y, withdraw) = match p {
            LiquidityPair::X => {
                let x2 = self.x.checked_add(a2).ok_or(ErrorCode::Overflow)?;
                let y2 = Self::y2_from_x_swap_amount(self.x, self.y, a2)?;
                let delta_y = Self::delta_y_from_x_swap_amount(self.x, self.y, a2)?;
                (x2, y2, delta_y)
            }
            LiquidityPair::Y => {
                let x2 = Self::x2_from_y_swap_amount(self.x, self.y, a)?;
                let y2 = self.y.checked_add(a).ok_or(ErrorCode::Overflow)?;
                let delta_x = Self::delta_x_from_y_swap_amount(self.x, self.y, a)?;
                (x2, y2, delta_x)
            }
        };

        Self::swap_slippage(withdraw, min)?;
        let fee = a.checked_sub(a2).ok_or(ErrorCode::Underflow)?;
        self.x = new_x;
        self.y = new_y;
    
        Ok(SwapResponse {
            deposit: a,
            fee,
            withdraw,
        })
    }
}
