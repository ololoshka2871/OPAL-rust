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

    /*
    /// Set the path to freeRTOS source
    /// Default is loaded from ENV variable "FREERTOS_SRC"
    pub fn freertos<P: AsRef<Path>>(&mut self, path: P) {
        self.freertos_dir = path.as_ref().to_path_buf();
    }
    /// Set the path to freeRTOSConfig.h
    /// Default is loaded from ENV variable, see: ENV_KEY_FREERTOS_CONFIG
    pub fn freertos_config<P: AsRef<Path>>(&mut self, path: P) {
        self.freertos_config_dir = path.as_ref().to_path_buf();
    }

    /// Set the path to shim.c (required for freertos-rust)
    /// Default is loaded from ENV variable, see: ENV_KEY_FREERTOS_SHIM
    pub fn freertos_shim<P: AsRef<Path>>(&mut self, path: P) {
        self.freertos_shim = path.as_ref().to_path_buf();
    }

    /// Returns a list of all files in the shim folder
    fn freertos_shim_files(&self) -> Vec<String> {
        let files: Vec<_> = WalkDir::new(self.freertos_shim.as_path())
            .follow_links(false)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter_map(|entry| {
                let f_name = entry.path().to_str().unwrap();

                if f_name.ends_with(".c") {
                    return Some(String::from(entry.path().to_str().unwrap()));
                }
                return None;
            }).collect();
        files
    }

    /// Returns a list of all FreeRTOS source files
    fn freertos_files(&self) -> Vec<String> {
        let files: Vec<_> = WalkDir::new(self.freertos_dir.as_path())
            .follow_links(false)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter_map(|entry| {
                let f_name = entry.path().to_str().unwrap();

                if f_name.ends_with(".c") {
                    return Some(String::from(entry.path().to_str().unwrap()));
                }
                return None;
            }).collect();
        files
    }
    fn freertos_port_files(&self) -> Vec<String> {
        let files: Vec<_> = WalkDir::new(self.get_freertos_port_dir())
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter_map(|entry| {
                let f_name = entry.path().to_str().unwrap();

                if f_name.ends_with(".c") {
                    return Some(String::from(entry.path().to_str().unwrap()));
                }
                return None;
            }).collect();
        files
    }

    /// Set the heap_?.c file to use from the "/portable/MemMang/" folder.
    /// heap_1.c ... heap_5.c (Default: heap_4.c)
    /// see also: https://www.freertos.org/a00111.html
    pub fn heap(&mut self, file_name: String) {
        self.heap_c = file_name;
    }

    /// Access to the underlining cc::Build instance to further customize the build.
    pub fn get_cc(&mut self) -> &mut Build {
        &mut self.cc
    }

    fn freertos_include_dir(&self) -> PathBuf {
        self.freertos_dir.join("include")
    }

    /// set the freertos port dir relativ to the FreeRTOS/Source/portable directory
    /// e.g. "GCC/ARM_CM33_NTZ/non_secure"
    ///
    /// If not set it will be detected based on the current build target (not many targets supported yet).
    pub fn freertos_port(&mut self, port_dir: String) {
        self.freertos_port = Some(port_dir);
    }

    fn get_freertos_port_dir(&self) -> PathBuf {
        let base = self.freertos_dir.join("portable");
        if self.freertos_port.is_some() {
            return base.join(self.freertos_port.as_ref().unwrap());
        }

        let target = env::var("TARGET").unwrap_or_default();
        let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default(); // msvc, gnu, ...
        //let target_family = env::var("CARGO_CFG_TARGET_FAMILY").unwrap_or_default(); // unix, windows
        let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default(); // x86_64
        let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default(); // none, windows, linux, macos
        let port = match (target.as_str(), target_arch.as_str(), target_os.as_str(), target_env.as_str()) {
            (_, "x86_64", "windows", _) => "MSVC-MingW",
            (_, "x86_64", "linux", "gnu") => "GCC/Linux",
            ("thumbv7m-none-eabi", _, _, _) => "GCC/ARM_CM3",
            ("thumbv7em-none-eabihf", _, _, _) => "GCC/ARM_CM4F",
            // TODO We should support feature "trustzone"
            ("thumbv8m.main-none-eabi", _, _, _) => "GCC/ARM_CM33_NTZ/non_secure",
            ("thumbv8m.main-none-eabihf", _, _, _) => "GCC/ARM_CM33_NTZ/non_secure",
            _ => {
                panic!("Unknown target: '{}', from TARGET environment variable.", target);
            }
        };
        return base.join(port);
    }

    fn heap_c_file(&self) -> PathBuf {
        self.freertos_dir.join("portable/MemMang").join(self.heap_c.as_str())
    }
    fn shim_c_file(&self) -> PathBuf {
        self.freertos_shim.join("shim.c")
    }
    */

    /*
    /// Check that all required files and paths exist
    fn verify_paths(&self) -> Result<(), Error> {
        if !self.freertos_dir.clone().exists() {
            return Err(Error::new(&format!("Directory freertos_dir does not exist: {}", self.freertos_dir.to_str().unwrap())));
        }
        let port_dir = self.get_freertos_port_dir();
        if !port_dir.clone().exists() {
            return Err(Error::new(&format!("Directory freertos_port_dir does not exist: {}", port_dir.to_str().unwrap())));
        }

        let include_dir = self.freertos_include_dir();
        if !include_dir.clone().exists() {
            return Err(Error::new(&format!("Directory freertos_include_dir does not exist: {}", include_dir.to_str().unwrap())));
        }

        // The heap implementation
        let heap_c = self.heap_c_file();
        if !heap_c.clone().exists() || !heap_c.clone().is_file() {
            return Err(Error::new(&format!("File heap_?.c does not exist: {}", heap_c.to_str().unwrap())));
        }

        // Allows to find the FreeRTOSConfig.h
        if !self.freertos_config_dir.clone().exists() {
            return Err(Error::new(&format!("Directory freertos_config_dir does not exist: {}", self.freertos_config_dir.to_str().unwrap())));
        }

        // Add the freertos shim.c to support freertos-rust
        let shim_c = self.shim_c_file();
        if !shim_c.clone().exists() || !shim_c.clone().is_file() {
            return Err(Error::new(&format!("File freertos_shim '{}' does not exist, missing freertos-rust dependency?", shim_c.clone().to_str().unwrap())));
        }

        Ok(())
    }
    */

    /*
    pub fn compile(&self) -> Result<(), Error> {
        let mut b = self.cc.clone();

        let path_error = self.verify_paths();
        if path_error.is_err() {
            return path_error;
        }

        // FreeRTOS header files
        b.include(self.freertos_include_dir());
        // FreeRTOS port header files (e.g. portmacro.h)
        b.include(self.get_freertos_port_dir());
        b.include(self.freertos_config_dir.clone());
        b.file(self.heap_c_file());
        self.freertos_files().iter().for_each(|f| {
            b.file(f);
        });
        self.freertos_port_files().iter().for_each(|f| {
            b.file(f);
        });
        self.freertos_shim_files().iter().for_each(|f| {
            b.file(f);
        });

        let res = b.try_compile("freertos");
        if res.is_err() {
            return Err(Error::new(&format!("{}", res.unwrap_err())));
        }

        Ok(())
    }
    */

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

    pub fn generate(self) -> () {
        self.verify_python3_exists();

        let (_dir, script) = Self::save_scripts();

        let mut args = vec![String::from(script.to_str().unwrap())];
        args.append(&mut self.generate_arguments());
        args.push(format!("--nanopb_out={}", env::var("OUT_DIR").unwrap()));

        println!(
            "cargo:warning=Executing generator: {} {}",
            self.python,
            args.join(" ")
        );

        let res = std::process::Command::new(self.python)
            .args(args)
            .output()
            .expect("Failed to start generator");

        if !res.status.success() {
            panic!(
                "Sources generete failed: {}",
                String::from_utf8(res.stderr).unwrap()
            );
        }

        ()
    }
}

/*
#[test]
fn test_paths() {
    env::set_var("FREERTOS_SRC", "some/path");
    env::set_var("TARGET", "thumbv8m.main-none-eabihf");
    let mut b = Builder::new();
    assert_eq!(b.freertos_dir.to_str().unwrap(), "some/path");
}*/
