use std::fs;

use crate::Error;

pub struct DirTreeNodes {
    name: String,
    num_subdir: i32,
    subdirs: Vec<DirTreeNodes>,
}

impl DirTreeNodes {
    pub fn getdirtree(path: String) -> Result<DirTreeNodes, Error> {
        match fs::read_dir(&path) {
            Ok(paths) => {
                let mut subdirs = Vec::new();

                for path in paths {
                    let path = path.unwrap();

                    let directory = path.path().to_string_lossy().into_owned();
                    match DirTreeNodes::getdirtree(directory) {
                        Ok(node) => subdirs.push(node),
                        Err(e) => match e.to_string().as_str() {
                            "Unable to find path" => {
                                continue;
                            }
                            _ => return Err(e),
                        },
                    }
                }

                let node = DirTreeNodes {
                    name: path,
                    num_subdir: subdirs.len() as i32,
                    subdirs,
                };
                Ok(node)
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Err("Unable to find path".into())
                } else {
                    Err(format!("Unknown error {}", e).into())
                }
            }
        }
    }

    /*
     * The Following struct should be prinicted in this format
     * Dir1:num_subdir
     * /tSubdir1:num_subdir
     * /t/tSubSubdir1:num_subdir
     * /t/t/tSubSubSubdir1:num_subdir
     * /t Subdir2:num_subdir
     */

    pub fn fmt_with_indent(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        indent: usize,
    ) -> std::fmt::Result {
        let pad = "\t".repeat(indent);
        writeln!(f, "{}{}:{}", pad, self.name, self.num_subdir)?;

        for dir in &self.subdirs {
            dir.fmt_with_indent(f, indent + 1)?;
        }

        Ok(())
    }
}
