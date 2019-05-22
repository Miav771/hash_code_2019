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
    number_of_tags: usize,
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
    number_of_tags: usize,
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

        let (_, smallest_waste_index) = slides
            .par_iter()
            .enumerate()
            .fold(
                || (usize::max_value(), 0),
                |(smallest_waste, smallest_waste_index), (index, slide)| {
                    let waste = calculate_waste(&current_slide, slide);
                    if waste < smallest_waste {
                        return (waste, index);
                    }
                    (smallest_waste, smallest_waste_index)
                },
            )
            .min_by_key(|(waste, _)| *waste)
            .unwrap();
        arranged_slides.push(current_slide);
        current_slide_index = smallest_waste_index;
    }
    arranged_slides
}

fn calculate_waste(left_slide: &Slide, right_slide: &Slide) -> usize {
    let common_tags = left_slide.tags.iter().fold(0, |intersection_count, tag| {
        intersection_count + right_slide.tags.contains(tag) as usize
    });
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

fn rate_slideshow(slides: &[Slide]) -> usize {
    let mut score = 0;
    for index in 0..slides.len() - 1 {
        let mut common_tags = 0;
        for tag in slides[index + 1].tags.iter() {
            if slides[index].tags.contains(&tag) {
                common_tags += 1;
            }
        }
        score += cmp::min(
            common_tags,
            cmp::min(
                slides[index + 1].number_of_tags,
                slides[index].number_of_tags,
            ) - common_tags,
        );
    }
    score
}

//Create slides from pictures
fn create_slides(mut pictures: Vec<Picture>) -> Vec<Slide> {
    let mut slides = Vec::new();
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
    while !vertical_pictures.is_empty() {
        let mut current_picture = vertical_pictures.pop().unwrap();
        let mut smallest_waste_index = 0;
        let mut smallest_waste = u32::max_value();
        for (index, picture) in vertical_pictures.iter().enumerate() {
            let mut waste = 0;
            for tag in current_picture.tags.iter() {
                if picture.tags.contains(tag) {
                    waste += 1;
                }
            }
            if waste < smallest_waste {
                smallest_waste = waste;
                smallest_waste_index = index;
            }
            if waste == 0 {
                break;
            }
        }
        let mut matching_picture = vertical_pictures.remove(smallest_waste_index);
        for tag in matching_picture.tags.drain(..) {
            if !current_picture.tags.contains(&tag) {
                current_picture.tags.push(tag);
            }
        }
        current_picture.tags.sort_unstable();
        slides.push(Slide {
            picture_id: current_picture.picture_id,
            second_picture_id: Option::Some(matching_picture.picture_id),
            orientation: Orientation::Vertical,
            number_of_tags: current_picture.tags.len(),
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
                //This will be populated with ids of tags, rather than actual tags
                //For now the tags for this picture will be kept as a second element of its tuple
                tags: Vec::new(),
            };
            let tags: Vec<String> = words.map(|tag| String::from(tag.trim())).collect();
            for tag in tags.iter() {
                all_tags.insert(tag.clone());
            }
            (picture, tags)
        })
        .collect();
    //Take all tags and assign a unique integer id to each
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
            picture.tags.sort_unstable();
            picture
        })
        .collect()
}
