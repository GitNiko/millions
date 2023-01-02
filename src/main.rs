
fn main() {
    // let parent = env::current_dir().unwrap();
    // let data_path = Path::new(&parent).join("example").join("gpcw20190630.dat");
    // let mut file = File::open(data_path).unwrap();
    // let mut buff = [0u8; HeaderSize];
    // file.read(&mut buff[..]).unwrap();
    // let header = Header::deserializer(&buff);
    // println!("{:?}", &header);

    // //https://github.com/rainx/pytdx/pull/150/files
    // file.seek(std::io::SeekFrom::Start(HeaderSize as u64));
    // println!(
    //     "seek: {:?}, header size: {:?}, align size: {:?}",
    //     std::io::SeekFrom::Start(size_of::<Header>().try_into().unwrap()),
    //     size_of::<Header>(),
    //     align_of::<Header>()
    // );
    // let mut stockBuff = [0u8; StockSize];
    // file.read(&mut stockBuff);
    // let stockHeader = StockItem::deserializer(&stockBuff);
    // // A6A0
    // println!("{:?}", &stockHeader);
    // let cols = header.report_size / 4;
    // println!("data cols count: {:?}", cols);
    // let col:u64 = 94;
    // file.seek(std::io::SeekFrom::Start(u64::from(stockHeader.offset) + col * 4));
    // let mut data_col = [0u8; StockFinanceColValueSize];
    // // let mut info_data = [0f32; 580];
    // file.read(&mut data_col);
    // let final_val = StockFinanceValue::deserializer(&data_col);
    // println!("data cols count: {:?}", final_val);

}

