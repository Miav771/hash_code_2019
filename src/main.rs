use rayon::prelude::*;
use std::cmp;
use std::collections::{HashMap, HashSet};
use std::fs;

const PROGRESS_REPORT_INTERVAL: usize = 10000;
const INPUT: &str = "abcde";

fn main() {
    process_inputs(INPUT);
}

#[derive(Debug)]
struct Slide {
    picture_id: usize,
    second_picture_id: Option<usize>,
    orientation: Orientation,
    number_of_tags: u32,
    tags: Vec<u32>,
}

#[derive(Debug)]
enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Debug)]
struct Picture {
    picture_id: usize,
    orientation: Orientation,
    number_of_tags: u32,
    tags: Vec<u32>,
}

fn process_inputs(params: &str) {
    for c in params.chars() {
        let pictures = parse_input(c);
        let slides = create_slides(pictures);
        let arranged_slides = arrange_slides(slides, c);
        let score = rate_slideshow(&arranged_slides);
        write_slides(&arranged_slides, format!("output_{}.txt", c).as_str());
        println!("Score for {}: {}", c, score);
    }
}

fn arrange_slides(mut slides: Vec<Slide>, name: char) -> Vec<Slide> {
    let mut arranged_slides: Vec<Slide> = Vec::with_capacity(slides.len());
    let mut current_slide_index = 0;
    while !slides.is_empty() {
        if slides.len() % PROGRESS_REPORT_INTERVAL == 0 {
            println!("Slides remaining for {}: {}", name, slides.len());
        }
        let current_slide = slides.remove(current_slide_index);
        current_slide_index = slides
            .par_iter()
            .enumerate()
            .min_by_key(|(_, potential_match)| calculate_waste(&current_slide, potential_match))
            .map(|(index, _)| index)
            .unwrap_or(0);
        arranged_slides.push(current_slide);
    }
    arranged_slides
}

fn calculate_common_tags(left_tags: &[u32], right_tags: &[u32]) -> u32 {
    let mut common_tags = 0;
    let mut left_iter = left_tags.iter();
    //Since the vectors are sorted, we can traverse each only once
    if let Some(mut left_tag) = left_iter.next() {
        'outer: for right_tag in right_tags.iter() {
            while left_tag < right_tag {
                left_tag = match left_iter.next() {
                    Some(left_tag) => left_tag,
                    None => break 'outer,
                };
            }
            if left_tag == right_tag {
                common_tags += 1;
            }
        }
    }
    common_tags
}

fn calculate_score(left_slide: &Slide, right_slide: &Slide) -> u32 {
    let common_tags = calculate_common_tags(&left_slide.tags, &right_slide.tags);
    let left_side = left_slide.number_of_tags - common_tags;
    let right_side = right_slide.number_of_tags - common_tags;
    cmp::min(common_tags, cmp::min(left_side, right_side))
}

fn calculate_waste(left_slide: &Slide, right_slide: &Slide) -> u32 {
    let common_tags = calculate_common_tags(&left_slide.tags, &right_slide.tags);
    let left_side = left_slide.number_of_tags - common_tags;
    let right_side = right_slide.number_of_tags - common_tags;
    let score = cmp::min(common_tags, cmp::min(left_side, right_side));
    left_side - score + right_side - score + common_tags - score
}

//Write output to file
fn write_slides(slides: &[Slide], filename: &str) {
    let mut output = String::new();
    output += format!("{}\n", slides.len()).as_str();
    for slide in slides {
        match slide.orientation {
            Orientation::Horizontal => output += format!("{}\n", slide.picture_id).as_str(),
            Orientation::Vertical => {
                output += format!(
                    "{} {}\n",
                    slide.picture_id,
                    slide.second_picture_id.unwrap()
                )
                .as_str()
            }
        }
    }
    fs::write(filename, output).unwrap();
}

fn rate_slideshow(slides: &[Slide]) -> u32 {
    slides.windows(2).fold(0, |score, slide_pair|{
        calculate_score(&slide_pair[0], &slide_pair[1]) + score
    })
}

//Create slides from pictures
fn create_slides(mut pictures: Vec<Picture>) -> Vec<Slide> {
    let mut slides = Vec::with_capacity(pictures.len()/2);
    let mut vertical_pictures = Vec::new();
    for picture in pictures.drain(..) {
        match picture.orientation {
            Orientation::Horizontal => {
                slides.push(Slide {
                    picture_id: picture.picture_id,
                    second_picture_id: Option::None,
                    orientation: Orientation::Horizontal,
                    number_of_tags: picture.number_of_tags,
                    tags: picture.tags.clone(),
                });
            }
            Orientation::Vertical => vertical_pictures.push(picture),
        }
    }
    vertical_pictures
        .sort_unstable_by(|x, y| y.number_of_tags.partial_cmp(&x.number_of_tags).unwrap());
    while let Some(mut current_picture) = vertical_pictures.pop() {
        let mut smallest_waste_index = 0;
        let mut smallest_waste = u32::max_value();
        for (index, picture) in vertical_pictures.iter().enumerate() {
            let waste = calculate_common_tags(&current_picture.tags, &picture.tags);
            if waste < smallest_waste {
                smallest_waste = waste;
                smallest_waste_index = index;
            }
            if waste == 0 {
                break;
            }
        }
        let mut matching_picture = vertical_pictures.remove(smallest_waste_index);
        current_picture.tags.append(&mut matching_picture.tags);
        current_picture.tags.dedup();
        current_picture.tags.sort_unstable();
        //Sort tags for faster future processing
        current_picture.tags.sort_unstable();
        slides.push(Slide {
            picture_id: current_picture.picture_id,
            second_picture_id: Option::Some(matching_picture.picture_id),
            orientation: Orientation::Vertical,
            number_of_tags: current_picture.tags.len() as u32,
            tags: current_picture.tags,
        })
    }
    slides
}

fn parse_input(input_number: char) -> (Vec<Picture>) {
    let input_name = match input_number {
        'a' => "a_example.txt",
        'b' => "b_lovely_landscapes.txt",
        'c' => "c_memorable_moments.txt",
        'd' => "d_pet_pictures.txt",
        'e' => "e_shiny_selfies.txt",
        _ => panic!("Wrong input"),
    };
    let file = fs::read_to_string(format!("inputs/{}", input_name))
        .expect("Couldn't find input files. Put input files in \"inputs\" folder");
    let lines = file.lines();
    let mut all_tags: HashSet<String> = HashSet::new();
    let mut picture_data: Vec<_> = lines
        //First line has no picture data in it
        .skip(1)
        .enumerate()
        .map(|(picture_number, line)| {
            let mut words = line.split(' ');
            let picture = Picture {
                picture_id: picture_number,
                orientation: match words.next().unwrap().trim() {
                    "H" => Orientation::Horizontal,
                    "V" => Orientation::Vertical,
                    _ => panic!("Wrong orientation"),
                },
                number_of_tags: words.next().unwrap().trim().parse().unwrap(),
                //This will be populated with integer representation of tags, rather than actual tags
                //For now the tags for this picture will be kept as a second element of a tuple
                tags: Vec::new(),
            };
            let tags: Vec<String> = words.map(|tag| String::from(tag.trim())).collect();
            for tag in tags.iter() {
                all_tags.insert(tag.clone());
            }
            (picture, tags)
        })
        .collect();
    //Map String tags to u32 integers
    let mut tag_map = HashMap::new();
    for (id, tag) in all_tags.drain().enumerate() {
        tag_map.insert(tag, id);
    }
    picture_data
        .drain(..)
        .map(|(mut picture, string_tags)| {
            picture.tags = string_tags
                .iter()
                .map(|tag| {
                    let id = tag_map[tag];
                    id as u32
                })
                .collect();
            //Sort tags for faster future processing
            picture.tags.sort_unstable();
            picture
        })
        .collect()
}
