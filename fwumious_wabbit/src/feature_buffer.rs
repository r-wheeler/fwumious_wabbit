use crate::model_instance;
use crate::record_reader;

const ONE:u32 = 1065353216;      // this is 1.0 float -> u32
const VOWPAL_FNV_PRIME:u32 = 16777619;	// vowpal magic number
const CONSTANT_NAMESPACE:usize = 128;
const CONSTANT_HASH:u32 = 11650396;


pub struct FeatureBuffer<'a> {
    model_instance: &'a model_instance::ModelInstance,
    pub output_buffer: Vec<u32>
}

impl<'a> FeatureBuffer<'a> {
    pub fn new(model_instance: &'a model_instance::ModelInstance) -> FeatureBuffer {
        let mut fb = FeatureBuffer{
                            model_instance: model_instance,
                            output_buffer: Vec::with_capacity(1024)
                        };
        fb
    }
    
    pub fn print(&self) -> () {
        println!("item out {:?}", self.output_buffer);
    }
    
    pub fn translate_vowpal(&mut self, record_buffer: &Vec<u32>) -> () {
        self.output_buffer.truncate(0);
        self.output_buffer.push(record_buffer[1]);
        let mut hashes_vec_in:Vec<u32> = Vec::with_capacity(100);
        let mut hashes_vec_out: Vec<u32> = Vec::with_capacity(100);
        
        for feature_combo_desc in &self.model_instance.feature_combo_descs {
            let feature_combo_weight_u32 = (feature_combo_desc.weight).to_bits();
            hashes_vec_in.truncate(0);
            hashes_vec_out.truncate(0);
            hashes_vec_in.push(0); // we always start with an empty value before doing recombos
            for feature_index in &feature_combo_desc.feature_indices {
                let feature_index_offset = *feature_index *2 + record_reader::HEADER_LEN;
                let start = record_buffer[feature_index_offset] as usize;
                let end = record_buffer[feature_index_offset+1] as usize;
                /*if start == 0 {
                    // When there is no value, we take it as special value 666+index feature
                    // This is so combo a,b,c wih b empty is different than a,c
                    // and it means missing a is different from missing b
                    let h = (feature_index_offset + 666) as u32;
                    for old_hash in &hashes_vec_in {
                        hashes_vec_out.push((*old_hash).wrapping_add(h));
                    }
                    
                } else {*/
                {
                    //println!("F1: {} {}", start, end);
                    for hash_offset in start..end {
                        let h = record_buffer[hash_offset];
                        for old_hash in &hashes_vec_in {
                            // This is just a copy of what vowpal does
                            // Verified to produce the same result
                            // ... we could use this in general case too, it's not too expansive (the additional mul)
                            let half_hash = (*old_hash).overflowing_mul(VOWPAL_FNV_PRIME).0;
                            hashes_vec_out.push(h ^ half_hash);
//                            println!("Output hash: {}", h ^ half_hash);
                        }
                    }
                }
                hashes_vec_in.truncate(0);
                let mut tmp = hashes_vec_in;
                hashes_vec_in = hashes_vec_out;
                hashes_vec_out = tmp;
            }
            for hash in &hashes_vec_in {
                self.output_buffer.push(*hash);
                self.output_buffer.push(feature_combo_weight_u32)
            //self.output_buffer.extend(&hashes_vec_in);
            }
        }
        // add the constant
        if self.model_instance.add_constant_feature {
            self.output_buffer.push(CONSTANT_HASH);
            self.output_buffer.push(ONE)
        }
        //println!("X {:?}", self.output_buffer);
    }

    pub fn translate_fwumnious(&mut self, record_buffer: &Vec<u32>) -> () {
        self.output_buffer.truncate(0);
        self.output_buffer.push(record_buffer[1]);
        let mut hashes_vec_in:Vec<u32> = Vec::with_capacity(100);
        let mut hashes_vec_out: Vec<u32> = Vec::with_capacity(100);
        
        for feature_combo_desc in &self.model_instance.feature_combo_descs {
            let feature_combo_weight_u32 = (feature_combo_desc.weight).to_bits();
            hashes_vec_in.truncate(0);
            hashes_vec_out.truncate(0);
            hashes_vec_in.push(0); // we always start with an empty value before doing recombos
            for feature_index in &feature_combo_desc.feature_indices {
                let feature_index_offset = *feature_index *2 + record_reader::HEADER_LEN;
                let start = record_buffer[feature_index_offset] as usize;
                let end = record_buffer[feature_index_offset+1] as usize;
                /*if start == 0 {
                    // When there is no value, we take it as special value 666+index feature
                    // This is so combo a,b,c wih b empty is different than a,c
                    // and it means missing a is different from missing b
                    let h = (feature_index_offset + 666) as u32;
                    for old_hash in &hashes_vec_in {
                        hashes_vec_out.push((*old_hash).wrapping_add(h));
                    }
                    
                } else {*/
                {
                    //println!("F1: {} {}", start, end);
                    for hash_offset in start..end {
                        let h = record_buffer[hash_offset];
                        for old_hash in &hashes_vec_in {
                            hashes_vec_out.push((*old_hash).wrapping_add(h));
                        }
                    }
                }
                hashes_vec_in.truncate(0);
                let mut tmp = hashes_vec_in;
                hashes_vec_in = hashes_vec_out;
                hashes_vec_out = tmp;
            }
            for hash in &hashes_vec_in {
                self.output_buffer.push(*hash);
                self.output_buffer.push(feature_combo_weight_u32)
            //self.output_buffer.extend(&hashes_vec_in);
            }
        }
        // add the constant
        if self.model_instance.add_constant_feature {
            self.output_buffer.push(CONSTANT_HASH);
            self.output_buffer.push(ONE)
        }
        //println!("X {:?}", self.output_buffer);
    }
}
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    fn add_header(v2: Vec<u32>) -> Vec<u32> {
        let mut rr: Vec<u32> = vec![100, 1];
        rr.extend(v2);
        rr
    }

    #[test]
    fn test_constant() {
        let mut mi = model_instance::ModelInstance::new_empty().unwrap();
        mi.add_constant_feature = true;
        mi.feature_combo_descs.push(model_instance::FeatureComboDesc {
                                                        feature_indices: vec![0], 
                                                        weight: 1.0});
        
        let mut fb = FeatureBuffer::new(&mi);
        let rb = add_header(vec![0, 0]); // no feature
        fb.translate_vowpal(&rb);
        assert_eq!(fb.output_buffer, vec![1, 11650396, ONE]); // vw compatibility - no feature is no feature
    }
    
    #[test]
    fn test_single_once() {
        let mut mi = model_instance::ModelInstance::new_empty().unwrap();
        mi.add_constant_feature = false;
        mi.feature_combo_descs.push(model_instance::FeatureComboDesc {
                                                        feature_indices: vec![0], 
                                                        weight: 1.0});
        
        let mut fb = FeatureBuffer::new(&mi);
        let rb = add_header(vec![0, 0]); // no feature
        fb.translate_vowpal(&rb);
        assert_eq!(fb.output_buffer, vec![1]); // vw compatibility - no feature is no feature

        let rb = add_header(vec![4, 5, 0xfea]);
        fb.translate_vowpal(&rb);
        assert_eq!(fb.output_buffer, vec![1, 0xfea, ONE]);

        let rb = add_header(vec![4, 6, 0xfea, 0xfeb]);
        fb.translate_vowpal(&rb);
        assert_eq!(fb.output_buffer, vec![1, 0xfea, ONE, 0xfeb, ONE]);
    }

    #[test]
    fn test_single_twice() {
        let mut mi = model_instance::ModelInstance::new_empty().unwrap();        
        mi.add_constant_feature = false;
        mi.feature_combo_descs.push(model_instance::FeatureComboDesc {
                                                        feature_indices: vec![0], 
                                                        weight: 1.0});
        mi.feature_combo_descs.push(model_instance::FeatureComboDesc {
                                                        feature_indices: vec![1], 
                                                        weight: 1.0});

        let mut fb = FeatureBuffer::new(&mi);

        let rb = add_header(vec![0, 0, 0, 0]);
        fb.translate_vowpal(&rb);
        assert_eq!(fb.output_buffer, vec![1]);

        let rb = add_header(vec![6, 7, 0, 0, 0xfea]);
        fb.translate_vowpal(&rb);
        assert_eq!(fb.output_buffer, vec![1, 0xfea, ONE]);

        let rb = add_header(vec![6, 7, 7, 8, 0xfea, 0xfeb]);
        fb.translate_vowpal(&rb);
        assert_eq!(fb.output_buffer, vec![1, 0xfea, ONE, 0xfeb, ONE]);

    }

    // for singles, vowpal and fwumnious are the same
    // however for doubles theya are not
    #[test]
    fn test_double_vowpal() {
        let mut mi = model_instance::ModelInstance::new_empty().unwrap();        
        mi.add_constant_feature = false;
        mi.feature_combo_descs.push(model_instance::FeatureComboDesc {
                                                        feature_indices: vec![0, 1], 
                                                        weight: 1.0});
        
        let mut fb = FeatureBuffer::new(&mi);
        let rb = add_header(vec![0, 0, 0, 0]);
        fb.translate_vowpal(&rb);
        assert_eq!(fb.output_buffer, vec![1]);

        let rb = add_header(vec![6, 7, 0, 0, (0xfea as u32).overflowing_mul(VOWPAL_FNV_PRIME).0]);
        fb.translate_vowpal(&rb);
        assert_eq!(fb.output_buffer, vec![1]);	// since the other feature is missing - VW compatibility says no feature is here

        let rb = add_header(vec![6, 7, 7, 8, 2988156968, 2422381320]);
        fb.translate_vowpal(&rb);
//        println!("out {}, out mod 2^24 {}", fb.output_buffer[1], fb.output_buffer[1] & ((1<<24)-1));
        assert_eq!(fb.output_buffer, vec![1, 434843120, ONE]);
        
    }
    
    #[test]
    fn test_double_fwumnious() {
        let mut mi = model_instance::ModelInstance::new_empty().unwrap();        
        mi.add_constant_feature = false;
        mi.feature_combo_descs.push(model_instance::FeatureComboDesc {
                                                        feature_indices: vec![0, 1], 
                                                        weight: 1.0});
        
        let mut fb = FeatureBuffer::new(&mi);
        let rb = add_header(vec![0, 0, 0, 0]);
        fb.translate_fwumnious(&rb);
        assert_eq!(fb.output_buffer, vec![1]);

        let rb = add_header(vec![6, 7, 0, 0, 0xfea]);
        fb.translate_fwumnious(&rb);
        assert_eq!(fb.output_buffer, vec![1]);	// since the other feature is missing - VW compatibility says no feature is here

        
        let rb = add_header(vec![6, 8, 8, 10, 0xfea, 0xfeb, 0xfec, 0xfed]);
        fb.translate_fwumnious(&rb);
        assert_eq!(fb.output_buffer, vec![1, 0xfea+0xfec, ONE, 0xfeb+0xfec, ONE, 0xfea+0xfed, ONE, 0xfeb+0xfed, ONE]);
    }

    #[test]
    fn test_single_with_weight() {
        let mut mi = model_instance::ModelInstance::new_empty().unwrap();        
        mi.add_constant_feature = false;
        mi.feature_combo_descs.push(model_instance::FeatureComboDesc {
                                                        feature_indices: vec![0], 
                                                        weight: 2.0});
        
        let mut fb = FeatureBuffer::new(&mi);
        let rb = add_header(vec![4, 5, 0xfea]);
        fb.translate_vowpal(&rb);
        let two = 2.0_f32.to_bits();

        assert_eq!(fb.output_buffer, vec![1, 0xfea, two]);
    }


}

