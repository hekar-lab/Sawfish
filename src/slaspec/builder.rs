use std::fs::{File, copy, create_dir_all};
use std::io::Write;
use std::path::Path;

use super::globals::{ALIGNMENT, ENDIAN, RAM_SAPCE, REGISTER_SPACE};
use super::instructions::core::InstrFamilyBuilder;

use super::instructions::instr16::*;

pub struct SLASpecBuilder {
    ifams_16: Vec<InstrFamilyBuilder>,
    ifams_32: Vec<InstrFamilyBuilder>,
    ifams_64: Vec<InstrFamilyBuilder>,
}

impl SLASpecBuilder {
    pub fn new() -> Self {
        let mut ifams_16: Vec<InstrFamilyBuilder> = vec![
            // MAIN_16A
            nop16::instr_fam(),
            progctrl::instr_fam(),
            pushpopreg::instr_fam(),
            cc2dreg::instr_fam(),
            cachectrl::instr_fam(),
            cc2stat::instr_fam(),
            pushpopmult::instr_fam(),
            ccmv::instr_fam(),
            ccflag::instr_fam(),
            brcc::instr_fam(),
            ujump::instr_fam(),
            regmv::instr_fam(),
            alu2op::instr_fam(),
            ptr2op::instr_fam(),
            logi2op::instr_fam(),
            comp3op::instr_fam(),
            compi2op::instr_fam(),
            // MAIN_16B
            ldstpmod::instr_fam(),
            ldst::instr_fam(),
            ldp::instr_fam(),
            dspldst::instr_fam(),
            dagmodim::instr_fam(),
            dagmodik::instr_fam(),
        ];

        for ifam in ifams_16.iter_mut() {
            ifam.init_tokens_and_vars();
        }

        let mut ifams_32: Vec<InstrFamilyBuilder> = vec![];

        for ifam in ifams_32.iter_mut() {
            ifam.init_tokens_and_vars();
        }

        let mut ifams_64: Vec<InstrFamilyBuilder> = vec![];

        for ifam in ifams_64.iter_mut() {
            ifam.init_tokens_and_vars();
        }

        SLASpecBuilder {
            ifams_16,
            ifams_32,
            ifams_64,
        }
    }

    fn build_main_header() -> String {
        let mut header = String::new();

        header += &format!("define endian={};\n", ENDIAN);
        header += &format!("define alignment={};\n", ALIGNMENT);
        header += "\n";
        header += &format!(
            "define space {} type=ram_space size=4 default;\n",
            RAM_SAPCE
        );
        header += &format!(
            "define space {} type=register_space size=2;\n",
            REGISTER_SPACE
        );
        header += "\n";

        header
    }

    fn build_main_file(path: &Path) {
        let mut file = File::create(path).unwrap();
        file.write_all(Self::build_main_header().as_bytes())
            .unwrap();

        file.write_all("@include \"includes/registers.sinc\"\n\n".as_bytes())
            .unwrap();

        file.write_all("@include \"includes/instructions.sinc\"\n".as_bytes())
            .unwrap();
    }

    fn instr_file_inc(dir: &str, file: &str) -> String {
        format!("@include \"{}/{}\"\n", dir, file)
    }

    fn build_instrs(
        instrs: &Vec<InstrFamilyBuilder>,
        inc_dir: &Path,
        instr_str: &str,
        inc_file: &mut File,
    ) {
        let instr_dir = inc_dir.join(instr_str);

        create_dir_all(&instr_dir).unwrap();

        for ifam in instrs {
            let filename = format!("{}.sinc", ifam.name());
            let instr_path = instr_dir.join(&filename);
            inc_file
                .write_all(Self::instr_file_inc(instr_str, &filename).as_bytes())
                .unwrap();
            let mut instr_file = File::create(instr_path).unwrap();

            instr_file.write_all(ifam.build().as_bytes()).unwrap();
        }
    }

    pub fn build(&self, path: &Path) {
        if create_dir_all(path).is_err() {
            panic!("Output directory cannot be created")
        }

        Self::build_main_file(&path.join("blackfinplus.slaspec"));
        let inc_dir = path.join("includes");

        create_dir_all(&inc_dir).unwrap();

        copy("data/registers.sinc", inc_dir.join("registers.sinc")).unwrap();

        let mut instr_inc_file = File::create(inc_dir.join("instructions.sinc")).unwrap();

        instr_inc_file
            .write_all("## 16-bits instructions ##\n\n".as_bytes())
            .unwrap();

        Self::build_instrs(&self.ifams_16, &inc_dir, "instr16", &mut instr_inc_file);

        instr_inc_file
            .write_all("\n## 32-bits instructions ##\n\n".as_bytes())
            .unwrap();

        Self::build_instrs(&self.ifams_32, &inc_dir, "instr32", &mut instr_inc_file);

        instr_inc_file
            .write_all("\n## 64-bits instructions ##\n\n".as_bytes())
            .unwrap();

        Self::build_instrs(&self.ifams_64, &inc_dir, "instr64", &mut instr_inc_file);
    }
}
