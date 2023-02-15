use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Too many poll options")]
    TooManyOptions {},

    #[error("Insufficient Kuji provided, please send at least 1.")]
    BadDenom{},

    #[error("This address is not staked with kujiDAO.")]
    NotStaked{},

    #[error("Poll has already been closed.")]
    ClosedPoll{},

    #[error("Poll does not exist.")]
    NoPoll{},

    #[error("A strange problem occurred.")]
    WeirdProblem{},
}
