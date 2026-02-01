//! Helper functions for creating common transaction patterns

use rhxcore::protocol::{ErrorCode, Field, Transaction, TransactionType};

/// Create a server-initiated transaction (no reply expected)
///
/// Server-initiated transactions always have:
/// - flags: 0
/// - is_reply: false
/// - id: 0 (server-initiated)
/// - error_code: 0
/// - total_size: 0
/// - data_size: 0
pub fn create_server_transaction(
    transaction_type: TransactionType,
    fields: Vec<Field>,
) -> Transaction {
    Transaction {
        flags: 0,
        is_reply: false,
        transaction_type,
        id: 0, // Server-initiated transaction
        error_code: 0,
        total_size: 0,
        data_size: 0,
        fields,
    }
}

/// Create an error reply transaction
///
/// Error replies always have:
/// - Same transaction type as the request
/// - Same transaction ID as the request
/// - is_reply: true
/// - Empty fields vec
pub fn create_error_reply(request: &Transaction, error_code: ErrorCode) -> Transaction {
    Transaction {
        flags: 0,
        is_reply: true,
        transaction_type: request.transaction_type,
        id: request.id,
        error_code: error_code.to_u32(),
        total_size: 0,
        data_size: 0,
        fields: vec![],
    }
}

/// Create a successful reply transaction
///
/// Success replies are like error replies but with error_code: 0
pub fn create_success_reply(request: &Transaction, fields: Vec<Field>) -> Transaction {
    Transaction {
        flags: 0,
        is_reply: true,
        transaction_type: request.transaction_type,
        id: request.id,
        error_code: 0,
        total_size: 0,
        data_size: 0,
        fields,
    }
}
