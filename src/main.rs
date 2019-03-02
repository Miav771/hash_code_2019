use std::cmp;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::thread;

fn main() {
    test("abcde");
}

#[derive(Debug)]
struct Slide {
    picture_id: u32,
    second_picture_id: Option<u32>,
    orientation: Orientation,
    number_of_tags: u8,
    tags: Vec<u32>,
}

#[derive(Debug)]
enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Debug)]
struct Picture {
    picture_id: u32,
    orientation: Orientation,
    number_of_tags: u8,
    tags: Vec<u32>,
}

fn test(params: &str) {
    let mut threads = Vec::new();
    for c in params.chars() {
        let thread_handle = thread::spawn(move || {
            let (number_of_pictures, tag_popularity_vector, pictures) = parse_input(&c);
            let slides = create_slides(pictures);
            let sorted_slides = sort_slides(slides);
            let arranged_slides = arrange_slides(sorted_slides, &c);
            let score = rate_slideshow(&arranged_slides);
            println!("Score for {}: {}", c, score);
            write_slides(arranged_slides, format!("output_{}.txt", c).as_str());
        });
        threads.push(thread_handle);
    }
    for thread_handle in threads {
        thread_handle.join().unwrap();
    }
}

fn arrange_slides(mut slides: Vec<Slide>, name: &char) -> Vec<Slide> {
    let mut arranged_slides: Vec<Slide> = Vec::with_capacity(slides.len());
    let mut current_slide_index = 0;
    let mut report = false;
    if *name == 'b' {
        report = true;
    }
    while !slides.is_empty() {
        if report && slides.len() % 5000 == 0 {
            println!("Slides remaining for {}: {}", name, slides.len());
        }
        let current_slide = slides.remove(current_slide_index);
        let mut smallest_waste = u8::max_value();
        let mut smallest_waste_index = 0;
        for index in 0..slides.len() {
            let mut common_tags = 0;
            for tag in current_slide.tags.iter() {
                if slides[index].tags.contains(&tag) {
                    common_tags += 1;
                }
            }
            let left_side = current_slide.number_of_tags - common_tags;
            let right_side = slides[index].number_of_tags - common_tags;
            let score = cmp::min(common_tags, cmp::min(left_side, right_side));
            let waste = left_side - score + right_side - score + common_tags - score;
            if waste < smallest_waste {
                smallest_waste = waste;
                smallest_waste_index = index;
            }
            if waste == 0 {
                break;
            }
        }
        arranged_slides.push(current_slide);
        current_slide_index = smallest_waste_index;
    }
    arranged_slides
}

fn sort_slides(mut slides: Vec<Slide>) -> Vec<Slide> {
    slides.sort_unstable_by(|x, y| y.number_of_tags.partial_cmp(&x.number_of_tags).unwrap());
    slides
}

fn write_slides(slides: Vec<Slide>, filename: &str) {
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

fn rate_slideshow(slides: &Vec<Slide>) -> u32 {
    let mut score: u32 = 0;
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
        ) as u32;
    }
    score
}

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
        let mut smallest_waste = u8::max_value();
        for index in 0..vertical_pictures.len() {
            let mut waste = 0;
            for tag in current_picture.tags.iter() {
                if vertical_pictures[index].tags.contains(tag) {
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
        slides.push(Slide {
            picture_id: current_picture.picture_id,
            second_picture_id: Option::Some(matching_picture.picture_id),
            orientation: Orientation::Vertical,
            number_of_tags: current_picture.tags.len() as u8,
            tags: current_picture.tags,
        })
    }
    slides
}

fn parse_input(input_number: &char) -> (u32, Vec<u32>, Vec<Picture>) {
    let input_name = match input_number {
        'a' => "a_example.txt",
        'b' => "b_lovely_landscapes.txt",
        'c' => "c_memorable_moments.txt",
        'd' => "d_pet_pictures.txt",
        'e' => "e_shiny_selfies.txt",
        _ => panic!("Wrong input"),
    };
    let file = fs::read_to_string(format!("inputs/{}", input_name)).unwrap();
    let mut lines = file.lines();
    let number_of_pictures: u32 = lines.next().unwrap().trim().parse().unwrap();
    let mut picture_number = 0;
    let mut all_tags: HashSet<String> = HashSet::new();
    let mut picture_data: Vec<_> = lines
        .map(|line| {
            let mut words = line.split(" ");
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
            picture_number += 1;
            (picture, tags)
        })
        .collect();
    //Take all tags and assign a unique integer id to each
    let mut tag_map = HashMap::new();
    let mut tag_popularity_vector = Vec::with_capacity(all_tags.len());
    for (id, tag) in all_tags.drain().enumerate() {
        tag_map.insert(tag, id);
        tag_popularity_vector.push(1);
    }
    let pictures = picture_data
        .drain(..)
        .map(|(mut picture, string_tags)| {
            picture.tags = string_tags
                .iter()
                .map(|tag| {
                    let id = tag_map.get(tag).unwrap().clone();
                    tag_popularity_vector[id] += 1;
                    id as u32
                })
                .collect();
            picture
        })
        .collect();
    (number_of_pictures, tag_popularity_vector, pictures)
}
