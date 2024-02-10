double lerp_inverse(double t, double min, double max) {
    return (t - min) / (max - min);
}

double map(double value, double from_source, double to_source, double from_target, double to_target) {
    return mix(from_target, to_target, lerp_inverse(value, from_source, to_source));
}

kernel void render(ulong width, double center_re, double center_im, double radius, uint max_iterations, global uint* output) {
    const size_t global_id = get_global_id(0);
    const size_t global_size = get_global_size(0);

    const ulong height = global_size / width;
    const double width_double = (double) width;    
    const double height_double = (double) height;

    const double max_offset_re = radius;
    const double max_offset_im = radius * height_double / width_double;

    const double bottom_left_re = center_re - max_offset_re;
    const double bottom_left_im = center_im - max_offset_im;
    const double top_right_re = center_re + max_offset_re;
    const double top_right_im = center_im + max_offset_im;

    const ulong x = global_id % width, y = height - global_id / width - 1;

    const double c_re = map(x, 0.0, width_double, bottom_left_re, top_right_re);
    const double c_im = map(y, 0.0, height_double, bottom_left_im, top_right_im);

    double z_re = 0.0;
    double z_im = 0.0;
    uint iteration;
    for (iteration = 0; iteration < max_iterations && z_re * z_re + z_im * z_im < 4.0; iteration++) {
        const double z_re_old = z_re;
        z_re = z_re * z_re - z_im * z_im + c_re;
        z_im = 2.0 * z_re_old * z_im + c_im;
    }
    output[global_id] = iteration;
}
