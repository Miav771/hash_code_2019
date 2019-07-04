use rayon::prelude::*;
use std::cmp;
use std::collections::HashMap;
use std::fs;

const PROGRESS_REPORT_INTERVAL: usize = 10000;
const INPUT: &str = "abcde";

fn main() {
    process_inputs(INPUT);
}

#[derive(Debug)]
struct Slide {
    picture_id: u32,
    second_picture_id: Option<u32>,
    tags: Vec<u32>,
}

#[derive(Debug)]
enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Debug)]
struct Picture {
    id: u32,
    orientation: Orientation,
    tags: Vec<u32>,
}

fn process_inputs(params: &str) {
    let mut total_score = 0;
    for c in params.chars() {
        let pictures = parse_input(c);
        let slides = create_slides(pictures);
        let arranged_slides = arrange_slides(slides, c);
        let score = rate_slideshow(&arranged_slides);
        total_score += score;
        write_slides(&arranged_slides, format!("output_{}.txt", c).as_str());
        println!("Score for {}: {}", c, score);
    }
    println!("Total score: {}", total_score);
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
            .min_by_key(|(_, potential_match)| {
                calculate_waste(&current_slide.tags, &potential_match.tags)
            })
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

fn calculate_score(left_tags: &[u32], right_tags: &[u32]) -> u32 {
    let common_tags = calculate_common_tags(left_tags, right_tags);
    let left_side = left_tags.len() as u32 - common_tags;
    let right_side = right_tags.len() as u32 - common_tags;
    cmp::min(common_tags, cmp::min(left_side, right_side))
}

fn calculate_waste(left_tags: &[u32], right_tags: &[u32]) -> u32 {
    let common_tags = calculate_common_tags(left_tags, right_tags);
    let left_side = left_tags.len() as u32 - common_tags;
    let right_side = right_tags.len() as u32 - common_tags;
    let score = cmp::min(common_tags, cmp::min(left_side, right_side));
    left_side - score + right_side - score + common_tags - score
}

//Write output to file
fn write_slides(slides: &[Slide], filename: &str) {
    let output = slides
        .iter()
        .fold(slides.len().to_string(), |output, slide| {
            if let Some(second_picture_id) = slide.second_picture_id {
                output + format!("\n{} {}", slide.picture_id, second_picture_id).as_str()
            } else {
                output + format!("\n{}", slide.picture_id).as_str()
            }
        });
    fs::write(filename, output).expect("Couldn't write output");
}

fn rate_slideshow(slides: &[Slide]) -> u32 {
    slides.windows(2).fold(0, |score, slide_pair| {
        calculate_score(&slide_pair[0].tags, &slide_pair[1].tags) + score
    })
}

//Create slides from pictures
fn create_slides(pictures: Vec<Picture>) -> Vec<Slide> {
    let (horizontal_pictures, mut vertical_pictures): (Vec<_>, Vec<_>) = pictures
        .into_iter()
        .partition(|picture| match picture.orientation {
            Orientation::Horizontal => true,
            Orientation::Vertical => false,
        });
    let mut slides: Vec<_> = horizontal_pictures
        .into_iter()
        .map(|picture| Slide {
            picture_id: picture.id,
            second_picture_id: None,
            tags: picture.tags,
        })
        .collect();
    vertical_pictures.sort_unstable_by_key(|picture| picture.tags.len());
    vertical_pictures.reverse();
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
        slides.push(Slide {
            picture_id: current_picture.id,
            second_picture_id: Option::Some(matching_picture.id),
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
    let mut tag_map = HashMap::new();
    lines
        .skip(1) //First line has no picture data in it
        .enumerate()
        .map(|(picture_number, line)| {
            let mut words = line.split_whitespace();
            let id = picture_number as u32;
            let orientation = match words.next().expect("Missing orientation information") {
                "H" => Orientation::Horizontal,
                "V" => Orientation::Vertical,
                _ => panic!("Invalid orientation"),
            };
            let mut tags: Vec<_> = words
                .skip(1) //The next word is the number of tags in a picture, which is known from the size of the Vec anyway
                .map(|tag| {
                    let numerical_tag = tag_map.len() as u32;
                    *tag_map.entry(tag).or_insert(numerical_tag)
                })
                .collect();
            //Sort tags to enable faster calculation of common_tags
            tags.sort_unstable();
            Picture {
                id,
                orientation,
                tags,
            }
        })
        .collect()
}
