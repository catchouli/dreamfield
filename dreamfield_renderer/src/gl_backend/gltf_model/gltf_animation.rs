use std::sync::{Arc, Mutex};
use cgmath::{VectorSpace, Vector3, vec3, Quaternion};
use gltf::animation::Property;
use byteorder::{LittleEndian, ReadBytesExt};
use super::{GltfTransform, GltfTransformHierarchy};

/// A gltf animation
pub struct GltfAnimation {
    name: String,
    length: f32,
    channels: Vec<GltfAnimationChannel> 
}

/// One channel of an animation
pub struct GltfAnimationChannel {
    target_node: Option<Arc<Mutex<GltfTransform>>>,
    frames: Vec<GltfAnimationKeyframe>
}

/// One frame of an animation channel
#[derive(Debug)]
pub enum GltfAnimationKeyframe {
    Translation(f32, Vector3<f32>),
    Rotation(f32, Quaternion<f32>),
    Scale(f32, Vector3<f32>)
}

impl GltfAnimation {
    /// Load an animation
    pub fn load(anim: &gltf::Animation, buffer_data: &[gltf::buffer::Data], hierarchy: &GltfTransformHierarchy)
        -> Self
    {
        log::debug!("Loading animation {}", anim.name().unwrap());

        // Get name
        let name = anim.name().unwrap_or(&format!("unnamed_{}", anim.index())).to_string();

        // Load channels
        let mut length = 0.0;
        let channels = anim.channels().map(|channel| {
            // Get target node
            let target = &channel.target();
            let target_node = hierarchy.node_by_index(target.node().index()).as_ref().map(Clone::clone);

            // Load frames
            let sampler = &channel.sampler();
            let (channel_length, frames) = Self::load_animation_frames(&sampler.input(), &sampler.output(),
                target.property(), &buffer_data);

            if channel_length > length {
                length = channel_length;
            }

            GltfAnimationChannel {
                target_node,
                frames
            }
        }).collect();

        GltfAnimation {
            name,
            length,
            channels
        }
    }

    /// Load animation frames
    fn load_animation_frames(input: &gltf::Accessor, output: &gltf::Accessor, property: Property,
        buffer_data: &[gltf::buffer::Data]) -> (f32, Vec<GltfAnimationKeyframe>)
    {
        const F32_SIZE: usize = std::mem::size_of::<f32>();

        // Get buffer view and data
        let frame_count = input.count();
        log::trace!("Loading {} {:?} animation frames", frame_count, property);

        let property_components = match property {
            Property::Translation => 3,
            Property::Rotation => 4,
            Property::Scale => 3,
            Property::MorphTargetWeights => panic!("not implemented")
        };

        let input_view = input.view().unwrap();
        let input_buffer_data = &buffer_data[input_view.buffer().index()];
        let input_buffer_length = frame_count * F32_SIZE;

        let output_view = output.view().unwrap();
        let output_buffer_data = &buffer_data[output_view.buffer().index()];
        let output_buffer_length = frame_count * property_components * F32_SIZE;

        // Safety checks
        assert!(input.data_type().size() == F32_SIZE);
        assert!(output.data_type().size() == F32_SIZE);
        assert!(input_view.length() == input_buffer_length);
        assert!(output_view.length() == output_buffer_length);

        // Load animation frames
        let mut input_reader = {
            let input_start = input_view.offset();
            let input_end = input_start + input_buffer_length;
            input_buffer_data.get(input_start..input_end).unwrap()
        };

        let mut output_reader = {
            let output_start = output_view.offset();
            let output_end = output_start + output_buffer_length;
            output_buffer_data.get(output_start..output_end).unwrap()
        };

        let mut res = Vec::new();
        let mut length = 0.0;

        for _ in 0..frame_count {
            let frame_time = input_reader.read_f32::<LittleEndian>().unwrap();

            let frame = match property {
                Property::Translation => {
                    GltfAnimationKeyframe::Translation(
                        frame_time,
                        vec3(
                            output_reader.read_f32::<LittleEndian>().unwrap(),
                            output_reader.read_f32::<LittleEndian>().unwrap(),
                            output_reader.read_f32::<LittleEndian>().unwrap()
                            ))
                },
                Property::Rotation => {
                    let x = output_reader.read_f32::<LittleEndian>().unwrap();
                    let y = output_reader.read_f32::<LittleEndian>().unwrap();
                    let z = output_reader.read_f32::<LittleEndian>().unwrap();
                    let w = output_reader.read_f32::<LittleEndian>().unwrap();

                    GltfAnimationKeyframe::Rotation(frame_time, Quaternion::new(w, x, y, z))
                },
                Property::Scale => {
                    GltfAnimationKeyframe::Scale(
                        frame_time,
                        vec3(
                            output_reader.read_f32::<LittleEndian>().unwrap(),
                            output_reader.read_f32::<LittleEndian>().unwrap(),
                            output_reader.read_f32::<LittleEndian>().unwrap()
                            ))
                },
                Property::MorphTargetWeights => panic!("not implemented")
            };

            if frame_time > length {
                length = frame_time;
            }

            res.push(frame);
        }

        // Check data is fully consumed
        assert!(input_reader.len() == 0);
        assert!(output_reader.len() == 0);

        (length, res)
    }


    /// Get animation name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the length of the animation
    pub fn length(&self) -> f32 {
        self.length
    }

    /// Get the channels
    pub fn channels(&self) -> &[GltfAnimationChannel] {
        &self.channels
    }
}

impl GltfAnimationChannel {
    /// Get target node
    pub fn target(&self) -> &Option<Arc<Mutex<GltfTransform>>> {
        &self.target_node
    }

    /// Sample the channel at a time
    pub fn sample(&self, time: f32) -> GltfAnimationKeyframe {
        let cur_frame = self.cur_frame(time);
        let total_frames = self.frames.len();

        let (left_frame, right_frame) = if cur_frame == 0 {
            (0, 0)
        }
        else if cur_frame == total_frames {
            (cur_frame - 1, cur_frame - 1)
        }
        else {
            (cur_frame - 1, cur_frame)
        };

        let left = &self.frames[left_frame];
        let right = &self.frames[right_frame];

        left.interpolate(right, time)
    }

    /// Perform a binary search to figure out the current animation frame
    fn cur_frame(&self, time: f32) -> usize {
        let mut min = 0;
        let mut max = self.frames.len() as i32 - 1;

        while min <= max {
            let mid = min + (max - min) / 2;
            let frame_time = self.frames[mid as usize].time();

            if frame_time == time {
                return mid as usize;
            }
            else if frame_time < time {
                min = mid + 1;
            }
            else {
                max = mid - 1;
            }
        }

        (max + 1) as usize
    }
}

impl GltfAnimationKeyframe {
    /// Get the keyframe time
    fn time(&self) -> f32 {
        match self {
            GltfAnimationKeyframe::Translation(time, _) => *time,
            GltfAnimationKeyframe::Rotation(time, _) => *time,
            GltfAnimationKeyframe::Scale(time, _) => *time
        }
    }

    /// Interpolate between two keyframes
    fn interpolate(&self, b: &GltfAnimationKeyframe, time: f32) -> GltfAnimationKeyframe {
        match (self, b) {
            (GltfAnimationKeyframe::Translation(t_a, p_a), GltfAnimationKeyframe::Translation(t_b, p_b)) => {
                let amount = Self::interpolation_amount(time, *t_a, *t_b);
                let position = p_a.lerp(*p_b, amount);
                GltfAnimationKeyframe::Translation(time, position)
            },
            (GltfAnimationKeyframe::Rotation(t_a, r_a), GltfAnimationKeyframe::Rotation(t_b, r_b)) => {
                let amount = Self::interpolation_amount(time, *t_a, *t_b);
                let rotation = r_a.slerp(*r_b, amount);
                GltfAnimationKeyframe::Rotation(time, rotation)
            },
            (GltfAnimationKeyframe::Scale(t_a, s_a), GltfAnimationKeyframe::Scale(t_b, s_b)) => {
                let amount = Self::interpolation_amount(time, *t_a, *t_b);
                let scale = s_a.lerp(*s_b, amount);
                GltfAnimationKeyframe::Scale(time, scale)
            }
            _ => panic!("Invalid combination of keyframes to interpolate")
        }
    }

    /// Get the interpolation amount for a time between two other times
    fn interpolation_amount(time: f32, a: f32, b: f32) -> f32 {
        f32::clamp((time - a) / (b - a), 0.0, 1.0)
    }
}

