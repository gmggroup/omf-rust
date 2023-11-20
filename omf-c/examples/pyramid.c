// Writes a small OMF file containing two elements: the surface and outline of a square pyramid.
// Then reads that file back and prints the data.

#include <omf.h>
#include <stdio.h>
#include <stdlib.h>

static const float VERTICES[][3] = {
    { -1.0, -1.0, 0.0 },
    {  1.0, -1.0, 0.0 },
    {  1.0,  1.0, 0.0 },
    { -1.0,  1.0, 0.0 },
    {  0.0,  0.0, 1.0 },
};

static const uint32_t TRIANGLES[][3] = {
    { 0, 1, 4 },
    { 1, 2, 4 },
    { 2, 3, 4 },
    { 3, 0, 4 },
    { 0, 2, 1 },
    { 0, 3, 2 },
};

const uint32_t SEGMENTS[][2] = {
    { 0, 1 },
    { 1, 2 },
    { 2, 3 },
    { 3, 0 },
    { 0, 4 },
    { 1, 4 },
    { 2, 4 },
    { 3, 4 },
};

static bool write(const char *path) {
    OmfError *error;
    OmfWriter *writer;
    OmfProject project;
    OmfSurface surface;
    OmfLineSet line_set;
    OmfElement element;
    const OmfArray *vertices;
    OmfHandle *proj_handle, *ele_handle, *tags_handle;

    omf_reader_limits(NULL);
    // Open file.
    writer = omf_writer_open(path);
    // Fill in `project` with the required name and optional description.
    project = omf_project_init("pyramid.omf");
    project.description = "Contains a square pyramid.";
    project.author = "Somebody";
    proj_handle = omf_writer_project(writer, &project);

    // First a surface element. Start writing the vertex and triangle arrays
    // and putting them in `surface`.
    vertices = omf_writer_array_vertices32(writer, VERTICES, 5);
    surface = omf_surface_init(
        vertices, omf_writer_array_triangles(writer, TRIANGLES, 6));
    // Fill in `element` with the surface and other fields.
    element = omf_element_init("Pyramid surface");
    element.surface = &surface;
    element.color_set = true;
    element.color[0] = 255;
    element.color[1] = 128;
    element.color[2] = 0;
    element.color[3] = 255; // Opaque
    // Write the element.
    ele_handle = omf_writer_element(writer, proj_handle, &element);
    // Add metadata to that element.
    omf_writer_metadata_string(writer, ele_handle, "revision", "1.2");
    tags_handle = omf_writer_metadata_list(writer, ele_handle, "tags");
    omf_writer_metadata_string(writer, tags_handle, NULL, "foo");
    omf_writer_metadata_string(writer, tags_handle, NULL, "bar");

    // Second a line-set element. This uses the same vertices array as the
    // surface. If we wrote it a second time the duplicate would be detected
    // and removed but we can also pass it in to both geometries.
    line_set = omf_line_set_init(
        vertices, omf_writer_array_segments(writer, SEGMENTS, 8));
    // Clear and fill in `element` again.
    element = omf_element_init("Pyramid outline");
    element.line_set = &line_set;
    element.color_set = true;
    element.color[0] = 0;
    element.color[1] = 0;
    element.color[2] = 0;
    element.color[3] = 128; // 50% transparent
    // And write it.
    omf_writer_element(writer, proj_handle, &element);

    // Finish writing and close the file.
    omf_writer_finish(writer, NULL);

    // Check for errors. The `omf_error` call will return the *first* error,
    // even if several functions failed since after detecting an invalid
    // argument.
    if ((error = omf_error()) != NULL) {
        fprintf(stderr, "[write failed] %s (%d)\n", error->message, error->code);
        fflush(stderr);
        omf_error_free(error);
        return false;
    }
    return true;
}

static bool read(const char *path) {
    OmfReader *reader;
    OmfError *error;
    const OmfProject *project;
    const OmfElement *e;
    // Dynamically allocated buffer for vertices.
    float (*vertices)[3];
    // For the triangles and segments we'll use fixed-size buffers for simplicity. Initialise
    // these buffers to zero so that if the read fails we don't end up printing uninitialised
    // memory.
    uint32_t segments[8][2] = { 0 };
    size_t i;
    OmfArrayInfo info;
    OmfTriangles *tri_iter;
    uint32_t tri[3];

    // Open the file.
    reader = omf_reader_open(path);
    // Read the project.
    project = omf_reader_project(reader, NULL);
    if (!project) {
        error = omf_error();
        fprintf(stderr, "[read failed] %s (%d)\n", error->message, error->code);
        omf_error_free(error);
        return false;
    }
    // Print project contents.
    printf("name: %s\n", project->name);
    printf("description: %s\n", project->description);
    printf("coordinate_reference_system: %s\n", project->coordinate_reference_system);
    printf("origin: %g, %g, %g\n", project->origin[0], project->origin[1], project->origin[2]);
    printf("author: %s\n", project->author);
    e = &project->elements[0];
    printf("surface:\n");
    printf("    name: %s\n", e->name);
    printf("    description: %s\n", e->description);
    printf("    color: #%02x%02x%02x%02x\n", e->color[0], e->color[1], e->color[2], e->color[3]);
    printf("    origin: %g, %g, %g\n",
           e->surface->origin[0], e->surface->origin[1], e->surface->origin[2]);
    // Allocate a buffer for the vertices, pretending we don't know the required length already.
    // Calloc Initializes the memory to zero for us.
    info = omf_reader_array_info(reader, e->surface->vertices);
    vertices = calloc(info.item_count, sizeof(float[3]));
    if (vertices == NULL) {
        fprintf(stderr, "memory allocation failed");
        return 1;
    }
    omf_reader_array_vertices32(reader, e->surface->vertices, vertices, info.item_count);
    printf("    vertices:\n");
    for (i = 0; i < info.item_count; i++) {
        printf("        % g, % g, % g\n", vertices[i][0], vertices[i][1], vertices[i][2]);
    }
    // Read the triangles using the iterator API.
    printf("    triangles:\n");
    tri_iter = omf_reader_array_triangles_iter(reader, e->surface->triangles);
    while (omf_triangles_next(tri_iter, tri)) {
        printf("        %d, %d, %d\n", tri[0], tri[1], tri[2]);
    }
    e = &project->elements[1];
    printf("line-set:\n");
    printf("    name: %s\n", e->name);
    printf("    description: %s\n", e->description);
    printf("    color: #%02x%02x%02x%02x\n", e->color[0], e->color[1], e->color[2], e->color[3]);
    printf("    origin: %g, %g, %g\n",
           e->line_set->origin[0], e->line_set->origin[1], e->line_set->origin[2]);
    // Read the segments into a fixed-size buffer.
    omf_reader_array_segments(reader, e->line_set->segments, segments, 8);
    printf("    segments:\n");
    for (i = 0; i < 8; i++) {
        printf("        %d, %d\n", segments[i][0], segments[i][1]);
    }

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

int main(void) {
    if (!write("pyramid.omf")) return 1;
    if (!read("pyramid.omf")) return 1;
    return 0;
}
