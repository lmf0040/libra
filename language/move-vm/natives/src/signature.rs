// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_crypto::{ed25519, traits::*};
use libra_types::vm_status::StatusCode;
use move_vm_types::{
    gas_schedule::NativeCostIndex,
    loaded_data::runtime_types::Type,
    natives::function::{native_gas, NativeContext, NativeResult},
    values::Value,
};
use std::{collections::VecDeque, convert::TryFrom};
use vm::errors::{PartialVMError, PartialVMResult};

/// Starting error code number
const DEFAULT_ERROR_CODE: u64 = 0x0ED2_5519;

pub fn native_ed25519_publickey_validation(
    context: &impl NativeContext,
    _ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(_ty_args.is_empty());
    debug_assert!(arguments.len() == 1);

    let key_bytes = pop_arg!(arguments, Vec<u8>);

    let cost = native_gas(
        context.cost_table(),
        NativeCostIndex::ED25519_VALIDATE_KEY,
        key_bytes.len(),
    );

    // This deserialization performs point-on-curve and small subgroup checks
    let valid = ed25519::Ed25519PublicKey::try_from(&key_bytes[..]).is_ok();
    let return_values = vec![Value::bool(valid)];
    Ok(NativeResult::ok(cost, return_values))
}

pub fn native_ed25519_signature_verification(
    context: &impl NativeContext,
    _ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(_ty_args.is_empty());
    debug_assert!(arguments.len() == 3);

    let msg = pop_arg!(arguments, Vec<u8>);
    let pubkey = pop_arg!(arguments, Vec<u8>);
    let signature = pop_arg!(arguments, Vec<u8>);

    let cost = native_gas(
        context.cost_table(),
        NativeCostIndex::ED25519_VERIFY,
        msg.len(),
    );

    let sig = match ed25519::Ed25519Signature::try_from(signature.as_slice()) {
        Ok(sig) => sig,
        Err(_) => {
            return Ok(NativeResult::err(
                cost,
                PartialVMError::new(StatusCode::NATIVE_FUNCTION_ERROR)
                    .with_sub_status(DEFAULT_ERROR_CODE),
            ));
        }
    };
    let pk = match ed25519::Ed25519PublicKey::try_from(pubkey.as_slice()) {
        Ok(pk) => pk,
        Err(_) => {
            return Ok(NativeResult::err(
                cost,
                PartialVMError::new(StatusCode::NATIVE_FUNCTION_ERROR)
                    .with_sub_status(DEFAULT_ERROR_CODE),
            ));
        }
    };

    let bool_value = sig.verify_arbitrary_msg(msg.as_slice(), &pk).is_ok();
    let return_values = vec![Value::bool(bool_value)];
    Ok(NativeResult::ok(cost, return_values))
}
