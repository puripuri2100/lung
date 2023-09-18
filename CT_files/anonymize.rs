//! cargo-scriptという拡張ソフトウェアを用いて実行することが前提。
//! `cargo install cargo-script`とすることでインストールできる。
//! 実行は`cargo script anonymize -- --input ja --output ja.json`のようにするだけでよい
//!
//! ```cargo
//! [package]
//! version = "0.1.0"
//! edition = "2021"
//!
//! [dependencies]
//! anyhow = "1.0.75"
//! clap = { version = "4.4.1", features = ["derive"] }
//! tokio = { version = "1.32.0", features = ["full"] }
//! tokio-stream = "0.1.14"
//! tracing = "0.1.37"
//! tracing-subscriber = "0.3.17"
//! dicom = "0.6.1"
//! dicom-core = "0.6.1"
//! rand = "0.8.5"
//! ```
//!

extern crate anyhow;
use anyhow::Result;

extern crate clap;
use clap::Parser;

extern crate tokio;

extern crate tokio_stream;
use tokio_stream::StreamExt;

extern crate tracing;
extern crate tracing_subscriber;
use tracing::*;

extern crate dicom;
extern crate dicom_core;
use dicom::object;
use dicom_core::header::{DataElement, Tag, VR};

extern crate rand;
use rand::prelude::*;

#[derive(Parser, Debug)]
#[command(version)]
struct AppArgs {
    #[arg(short, long)]
    input: String,
    #[arg(short, long)]
    output: String,
}

async fn init_logger() -> Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let app_args = AppArgs::parse();

    init_logger().await?;

    let mut rng = rand::thread_rng();

    let input_list = app_args.input.split(',').collect::<Vec<_>>();
    let output_list = app_args.output.split(',').collect::<Vec<_>>();
    let files = input_list.iter()
        .zip(output_list.iter())
        .collect::<Vec<(&&str, &&str)>>();
    let mut files_stream = tokio_stream::iter(files);

    while let Some((input, output)) = files_stream.next().await {
        info!("[START] {input}");
        info!("[START] read {input}");
        let mut obj = object::open_file(input)?;
        info!("[END] read {input}");

        // 標準DICOM画像タグセット一覧 - 医療用デジタル画像と通信タグ
        // https://www.liberworks.co.jp/know/know_dicomTag.html
        // タグの意味
        // https://www.ihe-j.org/file2/n13/1.2_DICOM_Tanaka.pdf
        // https://docs.rs/dicom-core/0.6.1/dicom_core/header/enum.VR.html

        // 患者氏名
        let old_patient_name = obj.element(Tag(0x0010, 0x0010))?.to_str()?;
        let new_patient_name = "puripuri^2100";
        info!("Patient Name: {old_patient_name} -> {new_patient_name}");
        let patient_name = DataElement::new(Tag(0x0010, 0x0010), VR::PN, new_patient_name);
        obj.put(patient_name);

        // 患者ID
        let old_patient_id = obj.element(Tag(0x0010, 0x0020))?.to_str()?;
        let new_patient_id = "0000123456";
        let patient_id = DataElement::new(Tag(0x0010, 0x0020), VR::LO, new_patient_name);
        info!("Patient ID: {old_patient_id} -> {new_patient_id}");
        obj.put(patient_id);

        // 患者の誕生日
        let old_patient_birth_date = obj.element(Tag(0x0010, 0x0030))?.to_str()?;
        let new_patient_birth_date = "200000401";
        let patient_birth_date =
            DataElement::new(Tag(0x0010, 0x0030), VR::DA, new_patient_birth_date);
        info!("Patient Birth Date: {old_patient_birth_date} -> {new_patient_birth_date}");
        obj.put(patient_birth_date);

        // 検査ID
        let old_study_id = obj.element(Tag(0x0020, 0x0010))?.to_str()?;
        let n: usize = rng.gen_range(0..100000000000);
        let new_study_id = format!("{n: >016}");
        let study_id = DataElement::new(Tag(0x0020, 0x0010), VR::SH, new_study_id.clone());
        info!("Study ID: {old_study_id} -> {new_study_id}");
        obj.put(study_id);

        // 施設名
        let old_institution_name = obj.element(Tag(0x0008, 0x0080))?.to_str()?;
        let new_institution_name = "FooBar Hospital";
        let institution_name = DataElement::new(Tag(0x0008, 0x0080), VR::LO, new_institution_name);
        info!("Institution Name: {old_institution_name} -> {new_institution_name}");
        obj.put(institution_name);

        info!("[START] write {output}");
        obj.write_to_file(output)?;
        info!("[END] write {output}");
        info!("[END] {input}");
    }

    Ok(())
}
