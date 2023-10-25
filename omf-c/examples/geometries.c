// Writes an OMF file containing one of each of the remaining element geometries.
// Surface and LineSet are covered in pyramid.c so won't be repeated here.

#include <assert.h>
#include <omf.h>
#include <stdio.h>
#include <stdlib.h>

// 2D Tensor data.
static const double TENSOR_U[] = { 2.0, 1.0 };
static const double TENSOR_V[] = { 1.0, 1.0 };
static const double TENSOR_W[] = { 0.5 };

// 2D grid heights.
static const float HEIGHTS[] = {
    -1.0, -1.0, -1.0,  -1.0, 1.0, -1.0,  -1.0, -1.0, -1.0,
};

// 2D vertices.
static const float VERTICES[][3] = {
    { 10.0, 0.0, -1.0 },
    { 12.0, 0.0, -1.0 },
    { 13.0, 0.0, -1.0 },
    { 10.0, 1.0, -1.0 },
    { 12.0, 1.0, 1.0 },
    { 13.0, 1.0, -1.0 },
    { 10.0, 2.0, -1.0 },
    { 12.0, 2.0, -1.0 },
    { 13.0, 2.0, -1.0 },
};

static const uint32_t REGULAR_SUBBLOCK_PARENTS[][3] = {
    { 0, 0, 0 },
    { 0, 0, 0 },
    { 0, 0, 0 },
    { 1, 0, 0 },
};

static const uint32_t REGULAR_SUBBLOCK_CORNERS[][6] = {
    { 0, 1, 0, 1, 2, 1 },
    { 1, 0, 0, 2, 1, 1 },
    { 1, 1, 0, 2, 2, 2 },
    { 0, 0, 0, 2, 2, 2 },
};

static const uint32_t FREEFORM_SUBBLOCK_PARENTS[][3] = {
    { 0, 0, 0 },
    { 0, 0, 0 },
    { 1, 0, 0 },
};

static const float FREEFORM_SUBBLOCK_CORNERS[][6] = {
    { 0.0, 0.0, 0.0, 0.5, 1.0, 0.17f },
    { 0.0, 0.0, 0.17f, 0.5, 1.0, 1.0 },
    { 0.0, 0.0, 0.0, 1.0, 1.0, 1.0 },
};

typedef struct {
    const float (*vertices)[3];
    size_t length;
    size_t index;
} VertexIter;

static bool next_vertex(void *object, double out[3]) {
    VertexIter *iter = object;
    if (iter->index >= iter->length) {
        return false;
    } else {
        out[0] = iter->vertices[iter->index][0];
        out[1] = iter->vertices[iter->index][1];
        out[2] = iter->vertices[iter->index][2];
        ++iter->index;
        return true;
    }
}

static bool write(const char *path) {
    OmfError *error;
    OmfWriter *writer;
    OmfHandle *proj_handle, *comp_handle;
    OmfProject project;
    OmfElement element;
    OmfPointSet point_set;
    OmfComposite composite;
    OmfGridSurface grid_surface;
    OmfTensorGrid2 tensor_grid2;
    OmfTensorGrid3 tensor_grid3;
    OmfRegularGrid3 regular_grid3;
    OmfBlockModel block_model;
    OmfRegularSubblocks subblocks;
    OmfFreeformSubblocks freeform_subblocks;
    VertexIter iter;

    // Open the file and create a project.
    writer = omf_writer_open(path);
    project = omf_project_init("geometries.omf");
    proj_handle = omf_writer_project(writer, &project);

    // Composite element, sub-elements added below.
    composite = omf_composite_init();
    element = omf_element_init("Container");
    element.composite = &composite;
    element.description = "Contains a grid surface, plus a point set of the vertices of that grid.";
    comp_handle = omf_writer_element(writer, proj_handle, &element);

    // GridSurface.
    tensor_grid2 = omf_tensor_grid2_init(
        omf_writer_array_scalars64(writer, TENSOR_U, 2),
        omf_writer_array_scalars64(writer, TENSOR_V, 2)
    );
    grid_surface = omf_grid_surface_init();
    grid_surface.orient.origin[0] = 10.0;
    grid_surface.tensor_grid = &tensor_grid2;
    grid_surface.heights = omf_writer_array_scalars32(writer, HEIGHTS, 9);
    element = omf_element_init("GridSurface");
    element.description = "An example 2D grid surface.";
    element.grid_surface = &grid_surface;
    omf_writer_element(writer, comp_handle, &element);

    // PointSet.
    // Write the vertices using the iterator API.
    iter.vertices = VERTICES;
    iter.index = 0;
    iter.length = 9;
    point_set = omf_point_set_init(omf_writer_array_vertices64_iter(writer, &next_vertex, &iter));
    element = omf_element_init("PointSet");
    element.description = "Points that should be in the same places as the grid vertices.";
    element.point_set = &point_set;
    omf_writer_element(writer, comp_handle, &element);

    // BlockModel with tensor grid and no sub-blocks.
    tensor_grid3 = omf_tensor_grid3_init(
        omf_writer_array_scalars64(writer, TENSOR_U, 2),
        omf_writer_array_scalars64(writer, TENSOR_V, 2),
        omf_writer_array_scalars64(writer, TENSOR_W, 1)
    );
    block_model = omf_block_model_init();
    block_model.tensor_grid = &tensor_grid3;
    element = omf_element_init("Tensor block model");
    element.block_model = &block_model;
    omf_writer_element(writer, proj_handle, &element);

    // BlockModel with regular sub-blocks.
    regular_grid3 = omf_regular_grid3_init(1.0, 1.0, 1.0, 2, 1, 1);
    subblocks = omf_regular_subblocks_init(
        2, 2, 2,
        omf_writer_array_regular_subblocks(writer, REGULAR_SUBBLOCK_PARENTS, REGULAR_SUBBLOCK_CORNERS, 4)
    );
    block_model = omf_block_model_init();
    block_model.regular_grid = &regular_grid3;
    block_model.regular_subblocks = &subblocks;
    element = omf_element_init("Regular block model with regular sub-blocks");
    element.block_model = &block_model;
    omf_writer_element(writer, proj_handle, &element);

    // BlockModel with free-form sub-blocks.
    regular_grid3 = omf_regular_grid3_init(1.0, 1.0, 1.0, 2, 1, 1);
    freeform_subblocks = omf_freeform_subblocks_init(
        omf_writer_array_freeform_subblocks32(writer, FREEFORM_SUBBLOCK_PARENTS, FREEFORM_SUBBLOCK_CORNERS, 3)
    );
    block_model = omf_block_model_init();
    block_model.regular_grid = &regular_grid3;
    block_model.freeform_subblocks = &freeform_subblocks;
    element = omf_element_init("Regular block model with free-form sub-blocks");
    element.block_model = &block_model;
    omf_writer_element(writer, proj_handle, &element);

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

static bool read(const char *path) {
    OmfReader *reader;
    const OmfProject *project;
    const OmfElement *element;
    OmfError *error;
    double u[2], v[2], heights[9], x, y, z;
    size_t i, j;
    double vertices[9][3];

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

    // Read and print the grid surface.
    element = &project->elements[0].composite->elements[0];
    printf("element: %s\n", element->name);
    omf_reader_array_scalars64(reader, element->grid_surface->tensor_grid->u, u, 2);
    omf_reader_array_scalars64(reader, element->grid_surface->tensor_grid->v, v, 2);
    // The heights were written as 'float' but can be read back as 'double'. Casting to larger
    // types within the same category (floating point, unsigned int, and signed int) is allowed.
    omf_reader_array_scalars64(reader, element->grid_surface->heights, heights, 9);
    y = element->grid_surface->orient.origin[1];
    for (j = 0; j <= 2; j++) {
        x = element->grid_surface->orient.origin[0];
        for (i = 0; i <= 2; i++) {
            z = heights[j * 3 + i] + element->grid_surface->orient.origin[2];
            printf("    %g %g %g\n", x, y, z);
            if (i < 2) {
                x += u[i];
            }
        }
        if (j < 2) {
            y += v[j];
        }
    }

    // Read and print the points.
    element = &project->elements[0].composite->elements[1];
    printf("element: %s\n", element->name);
    omf_reader_array_vertices64(reader, element->point_set->vertices, vertices, 9);
    for (i = 0; i < 9; i++) {
        printf("    %g %g %g\n", vertices[i][0], vertices[i][1], vertices[i][2]);
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

int main() {
    if (!write("geometries.omf")) return 1;
    if (!read("geometries.omf")) return 1;
    return 0;
}
