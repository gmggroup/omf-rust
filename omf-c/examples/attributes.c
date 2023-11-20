// Writes an OMF file containing all non-texture attributes on a cube surface, then reads
// back some of those attributes.

#include <assert.h>
#include <omf.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static const double VERTICES[][3] = {
    { 0.0, 0.0, 0.0 },
    { 1.0, 0.0, 0.0 },
    { 1.0, 1.0, 0.0 },
    { 0.0, 1.0, 0.0 },
    { 0.0, 0.0, 1.0 },
    { 1.0, 0.0, 1.0 },
    { 1.0, 1.0, 1.0 },
    { 0.0, 1.0, 1.0 },
};

static const uint32_t TRIANGLES[][3] = {
    { 0, 2, 1 },
    { 0, 3, 2 },
    { 0, 1, 5 },
    { 0, 5, 4 },
    { 1, 2, 6 },
    { 1, 6, 5 },
    { 2, 3, 7 },
    { 2, 7, 6 },
    { 3, 0, 4 },
    { 3, 4, 7 },
    { 4, 5, 6 },
    { 4, 6, 7 },
};

static const double PATH_VECTORS_3D[][3] = {
    {  1.0,  0.0,  0.0 },
    {  0.0,  1.0,  0.0 },
    { -1.0,  0.0,  0.0 },
    {  0.0,  0.0,  1.0 },
    {  0.0,  0.0, -1.0 },
    { -1.0,  0.0,  0.0 },
    {  0.0, -1.0,  0.0 },
    {  1.0,  0.0,  0.0 },
};

static const double OUTWARD_VECTORS_2D[][2] = {
    {  0.0,  0.0 },
    {  0.0,  0.0 },
    {  0.0, -1.0 },
    {  0.0, -1.0 },
    {  1.0,  0.0 },
    {  1.0,  0.0 },
    {  0.0,  1.0 },
    {  0.0,  1.0 },
    { -1.0,  0.0 },
    { -1.0,  0.0 },
    {  0.0,  0.0 },
    {  0.0,  0.0 },
};

static const bool OUTWARD_VECTORS_2D_MASK[] = {
    true, true, false, false, false, false, false, false, false, false, true, true,
};

static const bool FIRST_TRIANGLE[] = {
    true, false, true, false, true, false, true, false, true, false, true, false,
};

static const uint8_t COLORS[][4] = {
    {   0,   0,   0, 255 },
    { 255,   0,   0, 255 },
    { 255, 255,   0, 255 },
    {   0, 255,   0, 255 },
    {   0,   0, 255, 255 },
    { 255,   0, 255, 255 },
    { 255, 255, 255, 255 },
    {   0, 255, 255, 255 },
};

static const char *FACE_STRINGS[] = {
    "down", "down",
    "south", "south",
    "east", "east",
    "north", "north",
    "west", "west",
    "up", "up",
};

static const char *VERTEX_STRINGS[] = {
    "origin", NULL, NULL, NULL,
    NULL, NULL, NULL, NULL,
    NULL, NULL, NULL, NULL,
};

static const uint32_t CATEGORY_VALUES[] = {
    1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0,
};

static const char *CATEGORY_NAMES[] = {
    "ceiling",
    "floor",
    "wall",
};

static const int64_t CATEGORY_IDS[] = {
    1024,
    1025,
    -1,
};

static const uint8_t CATEGORY_COLORS[][4] = {
    { 255, 0, 0, 255 },
    { 0, 255, 0, 255 },
    { 0, 0, 255, 255 },
};

static const float NUMBERS[] = {
    0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0,
};

static const uint8_t GRADIENT[][4] = {
    { 255, 0, 0, 255 },
    { 255, 255, 0, 255 },
};

static const float DISCRETE_BOUNDARIES[] = {
    1.0, 4.0, 5.5, 7.5,
};
static const bool DISCRETE_INCLUSIVE[] = {
    true, // Includes the 1.0 value.
    false, // Excludes the 4.0 value.
    false,
    false,
};
static const uint8_t DISCRETE_COLORS[][4] = {
    { 255, 0, 0, 255 },
    { 255, 85, 0, 255 },
    { 255, 170, 0, 255 },
    { 255, 255, 0, 255 },
};

static const int64_t DATETIMES_MS[] = {
    -93706495806958LL,      // -1000-07-24T01:49:53.042
    -1465596606958LL,       //  1923-07-24T01:49:53.042
    1690163393042LL,        //  2023-07-24T01:49:53.042
    4845836993042LL,        //  2123-07-24T01:49:53.042
    32521312193042LL,       //  3000-07-24T01:49:53.042
    253388396993042LL,      //  9999-07-24T01:49:53.042
    0LL,                    //  1970-01-01T00:00:00.000 (the epoch)
    -2051264047219200000LL, // -65000000-01-01T00:00:00.000 (65 million years ago)
};

static bool write(const char *path) {
    OmfError *error;
    OmfWriter *writer;
    OmfProject project;
    OmfElement element;
    OmfSurface surface;
    OmfAttribute attribute;
    OmfCategoryData cat_data;
    OmfNumberData num_data;
    OmfContinuousColormap c_cmap;
    OmfDiscreteColormap d_cmap;
    OmfHandle *proj_handle, *cube_handle, *attr_handle;

    // Open the file and create a project.
    writer = omf_writer_open(path);
    project = omf_project_init("attributes.omf");
    proj_handle = omf_writer_project(writer, &project);

    // Create the cube element and keep that handle too.
    surface = omf_surface_init(
        omf_writer_array_vertices64(writer, VERTICES, 8),
        omf_writer_array_triangles(writer, TRIANGLES, 12)
    );
    element = omf_element_init("Cube");
    element.surface = &surface;
    cube_handle = omf_writer_element(writer, proj_handle, &element);

    // Masked 2D vectors on the faces. The attribute data is the array.
    attribute = omf_attribute_init("Outward", OMF_LOCATION_PRIMITIVES);
    attribute.description = "A vector on each face pointing outward in the XY plane, or null "
        "if the face is parallel to the XY plane.";
    attribute.vector_data = omf_writer_array_vectors64x2(
        writer, OUTWARD_VECTORS_2D, OUTWARD_VECTORS_2D_MASK, 12);
    omf_writer_attribute(writer, cube_handle, &attribute);

    // 3D vectors on the vertices. The attribute data is just the array for this type.
    attribute = omf_attribute_init("Path", OMF_LOCATION_VERTICES);
    attribute.description = "From each vertex, points toward the next vertex in a closed and "
        "non-intersecting path around the cube";
    attribute.vector_data = omf_writer_array_vectors64x3(writer, PATH_VECTORS_3D, NULL, 8);
    omf_writer_attribute(writer, cube_handle, &attribute);

    // Boolean values on faces. The attribute data is the array.
    attribute = omf_attribute_init("First triangle", OMF_LOCATION_PRIMITIVES);
    attribute.description = "Filter that selects the first triangle of each square face.";
    attribute.boolean_data = omf_writer_array_booleans(writer, FIRST_TRIANGLE, NULL, 12);
    omf_writer_attribute(writer, cube_handle, &attribute);

    // Color values on vertices. The attribute data is the array.
    attribute = omf_attribute_init("Position", OMF_LOCATION_VERTICES);
    attribute.description = "Transforms the vertex positions into RGB colors.";
    attribute.color_data = omf_writer_array_colors(writer, COLORS, NULL, 8);
    omf_writer_attribute(writer, cube_handle, &attribute);

    // Text values on faces. The attribute data is the string array.
    attribute = omf_attribute_init("Directions", OMF_LOCATION_PRIMITIVES);
    attribute.description = "Strings giving the direction of each face.";
    attribute.text_data = omf_writer_array_text(writer, FACE_STRINGS, 12);
    omf_writer_attribute(writer, cube_handle, &attribute);

    // Masked string values on vertices. The attribute data is the string array.
    attribute = omf_attribute_init("Origin", OMF_LOCATION_PRIMITIVES);
    attribute.description = "A string on just the origin vertex.";
    attribute.text_data = omf_writer_array_text(writer, VERTEX_STRINGS, 12);
    omf_writer_attribute(writer, cube_handle, &attribute);

    // Category values on faces. This is more complicated because we need to store the legend
    // as well.
    cat_data = omf_category_data_init();
    cat_data.values = omf_writer_array_indices(writer, CATEGORY_VALUES, NULL, 12);
    cat_data.names = omf_writer_array_names(writer, CATEGORY_NAMES, 3);
    cat_data.gradient = omf_writer_array_gradient(writer, CATEGORY_COLORS, 3);
    attribute = omf_attribute_init("Face type", OMF_LOCATION_PRIMITIVES);
    attribute.description = "The type of each face: wall, floor, or ceiling.";
    attribute.category_data = &cat_data;
    attr_handle = omf_writer_attribute(writer, cube_handle, &attribute);

    /// Add an integer sub-attribute to that category attribute.
    num_data = omf_number_data_init();
    num_data.values = omf_writer_array_numbers_int64(writer, CATEGORY_IDS, NULL, 3);
    attribute = omf_attribute_init("Discrete", OMF_LOCATION_CATEGORIES);
    attribute.description = "Category ids.";
    attribute.number_data = &num_data;
    omf_writer_attribute(writer, attr_handle, &attribute);

    // Number values on vertices with a discrete colormap.
    c_cmap = omf_continuous_colormap_init(
        0.0, 7.0, omf_writer_array_gradient(writer, GRADIENT, 2));
    num_data = omf_number_data_init();
    num_data.continuous_colormap = &c_cmap;
    num_data.values = omf_writer_array_numbers_float32(writer, NUMBERS, NULL, 8);
    attribute = omf_attribute_init("Continuous", OMF_LOCATION_VERTICES);
    attribute.description = "Numbers with a continuous colormap, shading from red to yellow.";
    attribute.number_data = &num_data;
    omf_writer_attribute(writer, cube_handle, &attribute);

    // Number values on vertices with a discrete colormap.
    d_cmap = omf_discrete_colormap_init();
    d_cmap.boundaries = omf_writer_array_boundaries_float32(
        writer, DISCRETE_BOUNDARIES, DISCRETE_INCLUSIVE, 4);
    d_cmap.gradient = omf_writer_array_gradient(writer, DISCRETE_COLORS, 5);
    num_data = omf_number_data_init();
    num_data.discrete_colormap = &d_cmap;
    num_data.values = omf_writer_array_numbers_float32(writer, NUMBERS, NULL, 8);
    attribute = omf_attribute_init("Discrete", OMF_LOCATION_VERTICES);
    attribute.description = "Numbers with a discrete colormap, shading from red to yellow with "
        "each color applied to two vertices.";
    attribute.number_data = &num_data;
    omf_writer_attribute(writer, cube_handle, &attribute);

    // Datetime values on vertices with no colormap.
    num_data = omf_number_data_init();
    num_data.values = omf_writer_array_numbers_date_time(writer, DATETIMES_MS, NULL, 8);
    attribute = omf_attribute_init("Date-times", OMF_LOCATION_VERTICES);
    attribute.description = "A scattering of date-time values as milliseconds since the epoch.";
    attribute.units = "datetime[ms]";
    attribute.number_data = &num_data;
    omf_writer_attribute(writer, cube_handle, &attribute);

    // Finish writing and close the file.
    omf_writer_finish(writer, NULL);

    // Check for errors.
    if ((error = omf_error()) != NULL) {
        fprintf(stderr, "[write failed] %s (%d)\n", error->message, error->code);
        omf_error_free(error);
        return false;
    }
    return true;
}

static void print_numbers_float32(OmfReader *reader, const OmfArray *array) {
    OmfArrayInfo info;
    OmfNumbersFloat32 *iter;
    float data;
    bool is_null;

    info = omf_reader_array_info(reader, array);
    assert(info.array_type == OMF_ARRAY_TYPE_NUMBERS_FLOAT32);
    iter = omf_reader_array_numbers_float32_iter(reader, array);
    while (omf_numbers_float32_next(iter, &data, &is_null)) {
        assert(!is_null);
        printf("    %g\n", data);
    }
    omf_numbers_float32_free(iter);
}

static void print_vectors64x2(OmfReader *reader, const OmfArray *array) {
    OmfArrayInfo info;
    OmfVectors64x2 *iter;
    double data[2];
    bool is_null;

    info = omf_reader_array_info(reader, array);
    assert(info.array_type == OMF_ARRAY_TYPE_VECTORS64X2);
    iter = omf_reader_array_vectors64x2_iter(reader, array);
    while (omf_vectors64x2_next(iter, data, &is_null)) {
        if (is_null) {
            printf("    null\n");
        } else {
            printf("    { %g, %g }\n", data[0], data[1]);
        }
    }
    omf_vectors64x2_free(iter);
}

static void print_text(OmfReader *reader, const OmfArray *array) {
    OmfArrayInfo info;
    OmfText *iter;
    const char *data;
    size_t len;

    info = omf_reader_array_info(reader, array);
    assert(info.array_type == OMF_ARRAY_TYPE_TEXT);
    iter = omf_reader_array_text_iter(reader, array);
    while (omf_text_next(iter, &data, &len)) {
        if (data == NULL) {
            assert(len == 0);
            printf("    null\n");
        } else {
            assert(len == strlen(data));
            printf("    \"%s\"\n", data);
        }
    }
    omf_text_free(iter);
}

static bool read(const char *path) {
    OmfReader *reader;
    const OmfProject *project;
    OmfError *error;
    const OmfAttribute *attribute;

    // Open the file and read the project.
    reader = omf_reader_open(path);
    project = omf_reader_project(reader, NULL);
    if (!project) {
        error = omf_error();
        fprintf(stderr, "[read failed] %s (%d)\n", error->message, error->code);
        omf_error_free(error);
        return false;
    }
    printf("name: %s\n", project->name);

    // Read a few of those attributes back and print them out.

    // Masked vector attribute.
    attribute = &project->elements[0].attributes[0];
    printf("%s:\n", attribute->name);
    print_vectors64x2(reader, attribute->vector_data);

    // String attribute.
    attribute = &project->elements[0].attributes[4];
    printf("%s:\n", attribute->name);
    print_text(reader, attribute->text_data);

    // Masked string attribute.
    attribute = &project->elements[0].attributes[5];
    printf("%s:\n", attribute->name);
    print_text(reader, attribute->text_data);

    // Number attribute.
    attribute = &project->elements[0].attributes[7];
    printf("%s:\n", attribute->name);
    print_numbers_float32(reader, attribute->number_data->values);

    // Close the reader only once we're done with `project`.
    omf_reader_close(reader);

    // Check for errors.
    if ((error = omf_error()) != NULL) {
        fprintf(stderr, "[read failed] %s (%d)\n", error->message,
                error->code);
        omf_error_free(error);
        return false;
    }
    return true;
}

int main() {
    if (!write("attributes.omf")) return 1;
    if (!read("attributes.omf")) return 1;
    return 0;
}
