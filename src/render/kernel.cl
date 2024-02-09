float lerp_inverse(float t, float min, float max) {
    return (t - min) / (max - min);
}

float map(float value, float from_source, float to_source, float from_target, float to_target) {
    return mix(from_target, to_target, lerp_inverse(value, from_source, to_source));
}

kernel void render(ulong width, float center_re, float center_im, float radius, uint max_iterations, global uint* output) {
    const size_t global_id = get_global_id(0);
    const size_t global_size = get_global_size(0);

    const ulong height = global_size / width;
    const float width_float = (float) width;    
    const float height_float = (float) height;

    const float max_offset_re = radius;
    const float max_offset_im = radius * height_float / width_float;

    const float bottom_left_re = center_re - max_offset_re;
    const float bottom_left_im = center_im - max_offset_im;
    const float top_right_re = center_re + max_offset_re;
    const float top_right_im = center_im + max_offset_im;

    const ulong x = global_id % width, y = height - global_id / width - 1;

    const float c_re = map(x, 0.0, width_float, bottom_left_re, top_right_re);
    const float c_im = map(y, 0.0, height_float, bottom_left_im, top_right_im);

    float z_re = 0.0;
    float z_im = 0.0;
    uint iteration;
    for (iteration = 0; iteration < max_iterations && z_re * z_re + z_im * z_im < 4.0; iteration++) {
        const float z_re_old = z_re;
        z_re = z_re * z_re - z_im * z_im + c_re;
        z_im = 2.0 * z_re_old * z_im + c_im;
    }
    output[global_id] = iteration;
}
