
pub trait Media {
    fn length() -> i32; // wait what aobut decimal chapters for manga fuck
    fn duration() -> i32;
}

// may also wanna move this into like a "models" module where i can have User as well and anything else that comes up

// #[derive(Clone, Default, Debug)]  // maybe can just read this in as a string ... ? and other things too... dunno if i should
// pub enum MediaFormat {
//     Tv,
//     TvShort,
//     Movie,
//     Special,
//     Ova,
//     Ona,
//     Music,
//     Manga,
//     Novel,
//     Oneshot,
// }

#[derive(Clone, Default, Debug)]
pub struct FuzzyDate {
    pub year: i32,
    pub month: i32,
    pub day: i32,
}

#[derive(Clone, Default, Debug)]
pub struct Title {
    pub romaji: String,
    pub english: String,
    pub native: String,
}

#[derive(Clone, Default, Debug)]
pub struct Anime {
    pub id: i32,
    pub title: Title,
    // pub format: MediaFormat,
    pub description: String,
    pub start_date: FuzzyDate,
    pub end_date: FuzzyDate,
    // pub season: 
}

#[derive(Clone, Default, Debug)]
pub struct Manga {

}

// impl Media for Anime {

// }