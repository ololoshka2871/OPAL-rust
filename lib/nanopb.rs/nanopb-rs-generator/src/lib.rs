use std::{
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use tempfile::TempDir;

static PYTHON_PROTOC_FILE: &[u8; 883] = include_bytes!("../../nanopb-dist/generator/protoc");
static PYTHON_GENERATOR_FILE: &[u8; 96302] =
    include_bytes!("../../nanopb-dist/generator/nanopb_generator.py");
static PYTHON_GEN_NANOPB: &[u8; 467] =
    include_bytes!("../../nanopb-dist/generator/protoc-gen-nanopb");
static PYTHON_GEN_NANOPB_BAT: &[u8; 449] =
    include_bytes!("../../nanopb-dist/generator/protoc-gen-nanopb.bat");
static PYTHON_PROTO_INIT_FILE: &[u8; 1127] =
    include_bytes!("../../nanopb-dist/generator/proto/__init__.py");
static PYTHON_PROTO__UTILS_FILE: &[u8; 1124] =
    include_bytes!("../../nanopb-dist/generator/proto/_utils.py");
static PYTHON_PROTO_NANOPB_PROTO: &[u8; 6611] =
    include_bytes!("../../nanopb-dist/generator/proto/nanopb.proto");
static PYTHON_PROTO_GOOGLE_PROTOBUF_DESCRIPTOR_PROTO: &[u8; 36277] =
    include_bytes!("../../nanopb-dist/generator/proto/google/protobuf/descriptor.proto");

#[derive(Clone, Debug)]
pub struct Generator {
    python: String,
    proto_file: Option<PathBuf>,
    add_proto_include_paths: Vec<PathBuf>,
}

impl Generator {
    /// Construct a new instance of a blank set of configuration.
    ///
    /// This builder is finished with the [`generate`] function.
    pub fn new() -> Self {
        Generator {
            python: "python3".to_string(),
            proto_file: None,
            add_proto_include_paths: vec![],
        }
    }

    pub fn set_python(mut self, executable: String) -> Self {
        self.python = executable;
        self
    }

    pub fn add_proto_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.proto_file = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn add_proto_include_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.add_proto_include_paths
            .push(path.as_ref().to_path_buf());
        self
    }

    fn verify_python3_exists(&self) {
        let _res = std::process::Command::new(&self.python)
            .arg("-V")
            .output()
            .expect("Python3 not found");
    }

    fn save_scripts() -> (TempDir, PathBuf) {
        let script_root = tempfile::tempdir()
            .map_err(|e| panic!("Failed to create temp dir {}", e))
            .unwrap();

        let protoc_file_name: PathBuf;
        {
            protoc_file_name = script_root.path().join("protoc");
            let mut protoc_file = File::create(&protoc_file_name).unwrap();
            protoc_file.write_all(PYTHON_PROTOC_FILE).unwrap();
        }
        {
            let generator_file_name = script_root.path().join("nanopb_generator.py");
            let mut generator_file = File::create(&generator_file_name).unwrap();
            generator_file.write_all(PYTHON_GENERATOR_FILE).unwrap();

            #[cfg(not(windows))]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&generator_file_name, fs::Permissions::from_mode(0o755))
                    .unwrap();
            }
        }
        {
            let protoc_gen_file_name = script_root.path().join("protoc-gen-nanopb");
            let mut protoc_gen_file = File::create(&protoc_gen_file_name).unwrap();
            protoc_gen_file.write_all(PYTHON_GEN_NANOPB).unwrap();

            #[cfg(not(windows))]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&protoc_gen_file_name, fs::Permissions::from_mode(0o755))
                    .unwrap();
            }
        }
        {
            let protoc_gen_file_bat_name = script_root.path().join("protoc-gen-nanopb.bat");
            let mut protoc_gen_file_bat = File::create(&protoc_gen_file_bat_name).unwrap();
            protoc_gen_file_bat
                .write_all(PYTHON_GEN_NANOPB_BAT)
                .unwrap();
        }

        {
            // proto
            let proto_path = script_root.path().join("proto");
            fs::create_dir(&proto_path).unwrap();

            {
                let init_py = proto_path.join("__init__.py");
                let mut init_py_file = File::create(init_py).unwrap();
                init_py_file.write_all(PYTHON_PROTO_INIT_FILE).unwrap();
            }
            {
                let utils_py = proto_path.join("_utils.py");
                let mut utils_py_file = File::create(utils_py).unwrap();
                utils_py_file.write_all(PYTHON_PROTO__UTILS_FILE).unwrap();
            }
            {
                let nanopb_proto = proto_path.join("nanopb.proto");
                let mut nanopb_proto_file = File::create(nanopb_proto).unwrap();
                nanopb_proto_file
                    .write_all(PYTHON_PROTO_NANOPB_PROTO)
                    .unwrap();
            }
            // proto/google/protobuf
            {
                let google_protobuf_path = proto_path.join("google").join("protobuf");
                fs::create_dir_all(&google_protobuf_path).unwrap();

                let descriptor_proto = google_protobuf_path.join("descriptor.proto");
                let mut descriptor_proto_file = File::create(descriptor_proto).unwrap();
                descriptor_proto_file
                    .write_all(PYTHON_PROTO_GOOGLE_PROTOBUF_DESCRIPTOR_PROTO)
                    .unwrap();
            }
        }

        (script_root, protoc_file_name)
    }

    fn generate_arguments(&self) -> Vec<String> {
        if let Some(proto_file) = &self.proto_file {
            if !proto_file.exists() {
                panic!("File \"{}\" does not exists", proto_file.to_str().unwrap());
            }
            let mut inc_paths = self.add_proto_include_paths.clone();
            inc_paths.push(proto_file.parent().unwrap().to_path_buf());
            let mut args = inc_paths
                .iter()
                .map(|p| {
                    if !p.exists() || !p.is_dir() {
                        panic!(
                            "Path \"{}\" does not exists or not a dirrectory",
                            proto_file.to_str().unwrap()
                        );
                    }
                    format!("-I{}", p.to_str().unwrap())
                })
                .collect::<Vec<String>>();
            args.push(String::from(proto_file.to_str().unwrap()));
            args
        } else {
            panic!("No .proto files provided!");
        }
    }

    fn run_generator(&self, args: Vec<String>) {
        println!(
            "cargo:warning=Executing generator: {} {}",
            &self.python,
            args.join(" ")
        );

        let res = std::process::Command::new(&self.python)
            .args(args)
            .output()
            .expect("Failed to start generator");

        if !res.status.success() {
            panic!(
                "Sources generete failed: {}",
                String::from_utf8(res.stderr).unwrap()
            );
        }
    }

    fn get_result_file(&self) -> PathBuf {
        let poutpath = PathBuf::from(env::var("OUT_DIR").unwrap());

        if let Some(src) = &self.proto_file {
            poutpath.join(str::replace(
                src.file_name().unwrap().to_str().unwrap(),
                ".proto",
                ".pb.c",
            ))
        } else {
            unreachable!();
        }
    }

    pub fn generate(self) -> PathBuf {
        self.verify_python3_exists();

        let (_dir, script) = Self::save_scripts();

        let mut args = vec![String::from(script.to_str().unwrap())];
        args.append(&mut self.generate_arguments());
        args.push(format!("--nanopb_out={}", env::var("OUT_DIR").unwrap()));

        self.run_generator(args);

        self.get_result_file()
    }
}
