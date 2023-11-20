// Demonstrates OMF metadata storage and retrieval.

#include <omf.h>
#include <stdio.h>
#include <stdlib.h>

static bool write(const char *path) {
    OmfError *error;
    OmfWriter *writer;
    OmfProject project;
    OmfHandle *proj_handle, *array_handle, *object_handle;

    // Open file.
    writer = omf_writer_open(path);
    // Create project and keep the handle to it.
    project = omf_project_init("metadata.omf");
    proj_handle = omf_writer_project(writer, &project);

    // Add a metadata value of each simple type. This is added directly to the project, but
    // an element or attribute handle will work too. This all gets stored as a chunk of
    // arbitrary JSON data in the file. Attaching too much meaning to metadata values may
    // make your file less useful in other applications as they won't necessarily know what
    // it means.
    //
    // Metadata keys, and values when they're strings, must be UTF-8 encoded. ASCII is also
    // acceptable because it's a subset of UTF-8.

    // Null values store only the key. This can be used where just the presence of the key is
    // useful or where a value isn't known.
    omf_writer_metadata_null(writer, proj_handle, "version");
    // Boolean values store true or false.
    omf_writer_metadata_boolean(writer, proj_handle, "is_draft", true);
    // Number values store a double value.
    omf_writer_metadata_number(writer, proj_handle, "importance", 2.6);
    // String value. This could also be used to store date or date/time values, which should
    // be in ISO 8601 format.
    omf_writer_metadata_string(writer, proj_handle, "source", "omf example code");

    // We can also store arrays of metadata values. Items in an array can have different types.
    // The same `omf_writer_metadata_*` functions are used to append array items, but the `key`
    // argument is ignored and should be null.
    array_handle = omf_writer_metadata_list(writer, proj_handle, "list");
    omf_writer_metadata_string(writer, array_handle, NULL, "first value");
    omf_writer_metadata_string(writer, array_handle, NULL, "second value");
    omf_writer_metadata_number(writer, array_handle, NULL, 3);

    // Finally we have object values, which contain their own key/value pairs. This is a good
    // way to group and label application-specific data for example.
    object_handle = omf_writer_metadata_object(writer, proj_handle, "my-company");
    omf_writer_metadata_string(
        writer, object_handle, "project-uuid", "550e8400-e29b-41d4-a716-446655440000");
    omf_writer_metadata_string(
        writer, object_handle, "project-uri", "https://example.com/");
    omf_writer_metadata_string(
        writer, object_handle, "project-revision", "1.4.2");

    // Finish writing and close the file.
    omf_writer_finish(writer, NULL);

    // Check for errors. The `omf_error` call will return the *first* error, even if several
    // functions failed since after detecting an invalid argument.
    if ((error = omf_error()) != NULL) {
        fprintf(stderr, "[write failed] %s (%d)\n", error->message, error->code);
        omf_error_free(error);
        return false;
    }
    return true;
}

static void print_indent(int indent) {
    int i;

    for (i = 0; i < indent; i++) {
        printf("    ");
    }
}

static void print_metadata_value(const OmfValue *value, int indent, bool is_array_item) {
    size_t i;

    print_indent(indent);
    if (!is_array_item) {
        printf("\"%s\": ", value->name);
    }
    // First check the value type.
    switch (value->type) {
    case OMF_VALUE_TYPE_NULL:
        // No valid fields for null values.
        printf("null,\n");
        break;
    case OMF_VALUE_TYPE_BOOLEAN:
        // `boolean` field is valid.
        printf("%s,\n", value->boolean ? "true" : "false");
        break;
    case OMF_VALUE_TYPE_NUMBER:
        // `number` field is valid.
        printf("%g,\n", value->number);
        break;
    case OMF_VALUE_TYPE_STRING:
        // `string` field is valid.
        printf("\"%s\",\n", value->string);
        break;
    case OMF_VALUE_TYPE_LIST:
        // `values` and `n_values` fields are valid and contain ordered values.
        printf("[\n");
        for (i = 0; i < value->n_values; i++) {
            print_metadata_value(&value->values[i], indent + 1, true);
        }
        print_indent(indent);
        printf("],\n");
        break;
    default: // OMF_VALUE_TYPE_OBJECT
        // `values` and `n_values` fields are valid and contain named values.
        printf("{\n");
        for (i = 0; i < value->n_values; i++) {
            print_metadata_value(&value->values[i], indent + 1, false);
        }
        print_indent(indent);
        printf("},\n");
        break;
    }
}

static bool read(const char *path) {
    OmfReader *reader;
    OmfError *error;
    const OmfProject *project;
    size_t i;

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

    // Metadata is stored as a list of `OmfValue` structs in `project->metadata` with length
    // `project->n_metadata`. The order that values were written in is not preserved.
    // Inside `OmfValue` the `type` field stores the type of the value and defines which other
    // fields are valid. Unused fields will be zeroed.
    //
    // The `OmfElement` and `OmfAttribute` fields have matching metadata fields.
    printf("metadata: {\n");
    for (i = 0; i < project->n_metadata; i++) {
        print_metadata_value(&project->metadata[i], 1, false);
    }
    printf("}\n");

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
    if (!write("metadata.omf")) return 1;
    if (!read("metadata.omf")) return 1;
    return 0;
}
