
use huff_utils::prelude::*;

use crate::Codegen;

pub fn generate_builtin_function_call_code(
    bf: &BuiltinFunctionCall,
    bytes: &mut Vec<(usize, Bytes)>,
    offset: &mut usize
) -> Result<(), CodegenError> {
    match bf.kind {
        BuiltinFunctionKind::Codesize => {
            let ir_macro = if let Some(m) =
                contract.find_macro_by_name(bf.args[0].name.as_ref().unwrap())
            {
                m
            } else {
                tracing::error!(
                    target: "codegen",
                    "MISSING MACRO PASSED TO __codesize \"{}\"",
                    bf.args[0].name.as_ref().unwrap()
                );
                return Err(CodegenError {
                    kind: CodegenErrorKind::MissingMacroDefinition(
                        bf.args[0].name.as_ref().unwrap().to_string(), /* yuck */
                    ),
                    span: bf.span.clone(),
                    token: None,
                })
            };

            let res: BytecodeRes = match Codegen::macro_to_bytecode(
                ir_macro.clone(),
                contract,
                scope,
                *offset,
                mis,
            ) {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!(
                        target: "codegen",
                        "FAILED TO RECURSE INTO MACRO \"{}\"",
                        ir_macro.name
                    );
                    return Err(e)
                }
            };

            let size = format_even_bytes(format!(
                "{:02x}",
                (res.bytes.iter().map(|(_, b)| b.0.len()).sum::<usize>() / 2)
            ));
            let push_bytes = format!("{:02x}{}", 95 + size.len() / 2, size);

            *offset += push_bytes.len() / 2;
            bytes.push((starting_offset, Bytes(push_bytes)));
        }
        BuiltinFunctionKind::Tablesize => {
            let ir_table = if let Some(t) =
                contract.find_table_by_name(bf.args[0].name.as_ref().unwrap())
            {
                t
            } else {
                tracing::error!(
                    target: "codegen",
                    "MISSING TABLE PASSED TO __tablesize \"{}\"",
                    bf.args[0].name.as_ref().unwrap()
                );
                return Err(CodegenError {
                    kind: CodegenErrorKind::InvalidMacroInvocation(
                        bf.args[0].name.as_ref().unwrap().to_string(), /* yuck */
                    ),
                    span: bf.span.clone(),
                    token: None,
                })
            };

            let size = bytes32_to_string(&ir_table.size, false);
            let push_bytes = format!("{:02x}{}", 95 + size.len() / 2, size);

            if !utilized_tables.contains(&ir_table) {
                utilized_tables.push(ir_table);
            }

            *offset += push_bytes.len() / 2;
            bytes.push((starting_offset, Bytes(push_bytes)));
        }
        BuiltinFunctionKind::Tablestart => {
            // Make sure the table exists
            if let Some(t) = contract.find_table_by_name(bf.args[0].name.as_ref().unwrap())
            {
                table_instances.push(Jump {
                    label: bf.args[0].name.as_ref().unwrap().to_owned(),
                    bytecode_index: *offset,
                    span: bf.span.clone(),
                });
                if !utilized_tables.contains(&t) {
                    utilized_tables.push(t);
                }

                bytes.push((*offset, Bytes(format!("{}xxxx", Opcode::Push2))));
                *offset += 3;
            } else {
                tracing::error!(
                    target: "codegen",
                    "MISSING TABLE PASSED TO __tablestart \"{}\"",
                    bf.args[0].name.as_ref().unwrap()
                );
                return Err(CodegenError {
                    kind: CodegenErrorKind::InvalidMacroInvocation(
                        bf.args[0].name.as_ref().unwrap().to_string(),
                    ),
                    span: bf.span.clone(),
                    token: None,
                })
            }
        }
        BuiltinFunctionKind::FunctionSignature => {
            if bf.args.len() != 1 {
                tracing::error!(
                    target: "codegen",
                    "Incorrect number of arguments passed to __FUNC_SIG, should be 1: {}",
                    bf.args.len()
                );
                return Err(CodegenError {
                    kind: CodegenErrorKind::InvalidArguments(
                        format!(
                            "Incorrect number of arguments passed to __FUNC_SIG, should be 1: {}",
                            bf.args.len()
                        )
                    ),
                    span: bf.span.clone(),
                    token: None,
                })
            }

            if let Some(func) = contract
                .functions
                .iter()
                .find(|f| bf.args[0].name.as_ref().unwrap().eq(&f.name))
            {
                let push_bytes =
                    format!("{}{}", Opcode::Push4, hex::encode(func.signature));
                *offset += push_bytes.len() / 2;
                bytes.push((starting_offset, Bytes(push_bytes)));
            } else if let Some(s) = &bf.args[0].name {
                let mut signature = [0u8; 4]; // Only keep first 4 bytes
                hash_bytes(&mut signature, s);

                let push_bytes = format!("{}{}", Opcode::Push4, hex::encode(signature));
                *offset += push_bytes.len() / 2;
                bytes.push((starting_offset, Bytes(push_bytes)));
            } else {
                tracing::error!(
                    target: "codegen",
                    "MISSING FUNCTION INTERFACE PASSED TO __SIG: \"{}\"",
                    bf.args[0].name.as_ref().unwrap()
                );
                return Err(CodegenError {
                    kind: CodegenErrorKind::MissingFunctionInterface(
                        bf.args[0].name.as_ref().unwrap().to_string(),
                    ),
                    span: bf.span.clone(),
                    token: None,
                })
            }
        }
        BuiltinFunctionKind::EventHash => {
            if bf.args.len() != 1 {
                tracing::error!(
                    target: "codegen",
                    "Incorrect number of arguments passed to __EVENT_HASH, should be 1: {}",
                    bf.args.len()
                );
                return Err(CodegenError {
                    kind: CodegenErrorKind::InvalidArguments(
                        format!(
                            "Incorrect number of arguments passed to __EVENT_HASH, should be 1: {}",
                            bf.args.len()
                        )
                    ),
                    span: bf.span.clone(),
                    token: None,
                })
            }

            if let Some(event) = contract
                .events
                .iter()
                .find(|e| bf.args[0].name.as_ref().unwrap().eq(&e.name))
            {
                let hash = bytes32_to_string(&event.hash, false);
                let push_bytes = format!("{}{}", Opcode::Push32, hash);
                *offset += push_bytes.len() / 2;
                bytes.push((starting_offset, Bytes(push_bytes)));
            } else if let Some(s) = &bf.args[0].name {
                let mut hash = [0u8; 32];
                hash_bytes(&mut hash, s);

                let push_bytes = format!("{}{}", Opcode::Push32, hex::encode(hash));
                *offset += push_bytes.len() / 2;
                bytes.push((starting_offset, Bytes(push_bytes)));
            } else {
                tracing::error!(
                    target: "codegen",
                    "MISSING EVENT INTERFACE PASSED TO __EVENT_HASH: \"{}\"",
                    bf.args[0].name.as_ref().unwrap()
                );
                return Err(CodegenError {
                    kind: CodegenErrorKind::MissingEventInterface(
                        bf.args[0].name.as_ref().unwrap().to_string(),
                    ),
                    span: bf.span.clone(),
                    token: None,
                })
            }
        }
        BuiltinFunctionKind::Error => {
            if bf.args.len() != 1 {
                tracing::error!(
                    target: "codegen",
                    "Incorrect number of arguments passed to __ERROR, should be 1: {}",
                    bf.args.len()
                );
                return Err(CodegenError {
                    kind: CodegenErrorKind::InvalidArguments(format!(
                        "Incorrect number of arguments passed to __ERROR, should be 1: {}",
                        bf.args.len()
                    )),
                    span: bf.span.clone(),
                    token: None,
                })
            }

            if let Some(error) = contract
                .errors
                .iter()
                .find(|e| bf.args[0].name.as_ref().unwrap().eq(&e.name))
            {
                // Add 28 bytes to left-pad the 4 byte selector
                let selector =
                    format!("{}{}", hex::encode(error.selector), "00".repeat(28));
                let push_bytes = format!("{}{}", Opcode::Push32, selector);
                *offset += push_bytes.len() / 2;
                bytes.push((starting_offset, Bytes(push_bytes)));
            } else {
                tracing::error!(
                    target: "codegen",
                    "MISSING ERROR DEFINITION PASSED TO __ERROR: \"{}\"",
                    bf.args[0].name.as_ref().unwrap()
                );
                return Err(CodegenError {
                    kind: CodegenErrorKind::MissingErrorDefinition(
                        bf.args[0].name.as_ref().unwrap().to_string(),
                    ),
                    span: bf.span.clone(),
                    token: None,
                })
            }
        }
        BuiltinFunctionKind::RightPad => {
            if bf.args.len() != 1 {
                tracing::error!(
                    target = "codegen",
                    "Incorrect number of arguments passed to __RIGHTPAD, should be 1: {}",
                    bf.args.len()
                );
                return Err(CodegenError {
                    kind: CodegenErrorKind::InvalidArguments(format!(
                        "Incorrect number of arguments passed to __RIGHTPAD, should be 1: {}",
                        bf.args.len()
                    )),
                    span: bf.span.clone(),
                    token: None,
                })
            }

            let hex = format_even_bytes(bf.args[0].name.as_ref().unwrap().clone());
            let push_bytes =
                format!("{}{}{}", Opcode::Push32, hex, "0".repeat(64 - hex.len()));
            *offset += push_bytes.len() / 2;
            bytes.push((starting_offset, Bytes(push_bytes)));
        }
    }

    return Ok(())
}