use std::fs::{File, copy, create_dir_all};
use std::io::Write;
use std::path::Path;

use super::globals::{ALIGNMENT, ENDIAN, RAM_SAPCE, REGISTER_SPACE};
use super::instructions::core::InstrFamilyBuilder;

use super::instructions::instr16::*;
use super::instructions::instr32::*;
use super::instructions::instr64::*;

pub struct SLASpecBuilder {
    ifams_16: Vec<InstrFamilyBuilder>,
    ifams_32: Vec<InstrFamilyBuilder>,
    ifams_64: Vec<InstrFamilyBuilder>,
}

impl SLASpecBuilder {
    pub fn new() -> Self {
        let mut instr_count = 0;
        let mut instr_total = 0;
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
            dspldst::instr_fam(),
            dagmodim::instr_fam(),
            dagmodik::instr_fam(),
            ldstii::instr_fam(),
            ldstiifp::instr_fam(),
        ];

        println!("Init 16-bits instructions...");
        for ifam in ifams_16.iter_mut() {
            ifam.init_tokens_and_vars();
            println!(
                "\t{:16} -> {:6} intruction(s)",
                ifam.name(),
                ifam.instrs().len()
            );
            instr_count += ifam.instrs().len();
        }
        println!("Count: {} 16-bits instructions\n", instr_count);
        instr_total += instr_count;
        instr_count = 0;

        let mut ifams_32: Vec<InstrFamilyBuilder> = vec![
            nop32::instr_fam(),
            dsp32mac::instr_fam(),
            dsp32mult::instr_fam(),
        ];

        println!("Init 32-bits instructions...");
        for ifam in ifams_32.iter_mut() {
            ifam.init_tokens_and_vars();
            println!(
                "\t{:16} -> {:6} intruction(s)",
                ifam.name(),
                ifam.instrs().len()
            );
            instr_count += ifam.instrs().len();
        }
        println!("Count: {} 32-bits instructions\n", instr_count);
        instr_total += instr_count;
        instr_count = 0;

        let mut ifams_64: Vec<InstrFamilyBuilder> = vec![];

        println!("Init 64-bits instructions...");
        for ifam in ifams_64.iter_mut() {
            ifam.init_tokens_and_vars();
            println!(
                "\t{:16} -> {:6} intruction(s)",
                ifam.name(),
                ifam.instrs().len()
            );
            instr_count += ifam.instrs().len();
        }
        println!("Count: {} 64-bits instructions\n", instr_count);
        instr_total += instr_count;

        println!("Intruction total: {}", instr_total);

        println!("INIT DONE :)\n");

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

        file.write_all("@include \"includes/instructions.sinc\"\n\n".as_bytes())
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

        println!("Building blackfinplus.slaspec...");
        Self::build_main_file(&path.join("blackfinplus.slaspec"));
        let inc_dir = path.join("includes");
        println!("DONE!\n");

        create_dir_all(&inc_dir).unwrap();

        println!("Copying registers.sinc...");
        copy("data/registers.sinc", inc_dir.join("registers.sinc")).unwrap();
        println!("DONE!\n");

        let mut instr_inc_file = File::create(inc_dir.join("instructions.sinc")).unwrap();

        println!("Building 16-bits instructions...");
        instr_inc_file
            .write_all("## 16-bits instructions ##\n\n".as_bytes())
            .unwrap();

        Self::build_instrs(&self.ifams_16, &inc_dir, "instr16", &mut instr_inc_file);

        println!("Building 32-bits instructions...");
        instr_inc_file
            .write_all("\n## 32-bits instructions ##\n\n".as_bytes())
            .unwrap();

        Self::build_instrs(&self.ifams_32, &inc_dir, "instr32", &mut instr_inc_file);

        println!("Building 64-bits instructions...");
        instr_inc_file
            .write_all("\n## 64-bits instructions ##\n\n".as_bytes())
            .unwrap();

        Self::build_instrs(&self.ifams_64, &inc_dir, "instr64", &mut instr_inc_file);
        println!("ALL DONE :3");
    }
}
