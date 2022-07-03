use crate::prelude::Artifact;
use std::{fs, path::Path, sync::Arc};

/// Generate solidity interfaces from a vector of artifacts.
///
/// @param artifacts The vector of artifacts to generate interfaces from.
/// @return The vector of generated interfaces.
pub fn gen_sol_interfaces(artifacts: &Vec<Arc<Artifact>>) -> Vec<(&str, String)> {
    let mut interfaces = Vec::new();

    for artifact in artifacts {
        if let Some(a) = &artifact.abi {
            let mut defs = Vec::new();
            a.events.iter().for_each(|(_, f)| {
                defs.push(format!(
                    "{}event {}({});",
                    "\t",
                    f.name,
                    f.inputs
                        .iter()
                        .map(|i| {
                            format!(
                                "{}{}",
                                i.kind,
                                if i.indexed {
                                    String::from(" indexed")
                                } else {
                                    String::default()
                                }
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", "),
                ));
            });
            a.functions.iter().for_each(|(_, f)| {
                defs.push(format!(
                    "{}function {}({}) external{}{};",
                    "\t",
                    f.name,
                    f.inputs
                        .iter()
                        .map(|i| format!(
                            "{}{}",
                            i.kind,
                            if i.kind.is_memory_type() { " memory" } else { "" }
                        ))
                        .collect::<Vec<_>>()
                        .join(", "),
                    f.state_mutability.interface_mutability(),
                    if f.outputs.is_empty() {
                        String::default()
                    } else {
                        format!(
                            " returns ({})",
                            f.outputs
                                .iter()
                                .map(|o| format!(
                                    "{}{}",
                                    o.kind,
                                    if o.kind.is_memory_type() { " memory" } else { "" }
                                ))
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    },
                ));
            });

            let interface_name =
                artifact.file.path.split('/').last().unwrap().split('.').next().unwrap();
            interfaces.push((
                interface_name,
                format!("interface I{} {{\n{}\n}}", interface_name, defs.join("\n"),),
            ));
        }
    }

    interfaces
}

/// Export generated solidity interfaces to a file.
///
/// @param interfaces The vector of generated interfaces.
/// @return Unit type if success, error if failure.
pub fn export_interfaces(interfaces: &Vec<(&str, String)>) -> Result<(), std::io::Error> {
    for (name, interface) in interfaces {
        let path_str = format!("./I{}.sol", name);
        let file_path = Path::new(&path_str);
        fs::write(file_path, interface)?;
    }
    Ok(())
}
