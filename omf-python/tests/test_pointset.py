import json
import omf_python
from os import path
from unittest import TestCase

class TestPointSet(TestCase):
    def setUp(self) -> None:
        omf_dir = path.join(path.dirname(__file__), "data")
        one_of_everything = path.join(omf_dir, "one_of_everything.omf")
        self.reader = omf_python.Reader(one_of_everything)

    def test_should_return_expected_geometry_type(self) -> None:
        pointset_type = self.reader.project.elements[1].geometry.type_name()

        self.assertEqual(pointset_type, "PointSet")

    def test_should_return_expected_origin(self) -> None:
        pointset_origin = self.reader.project.elements[1].geometry.get_object().origin

        self.assertEqual(pointset_origin, [0.0, 0.0, 0.0])

    def test_should_return_expected_vertices(self) -> None:
        # Given
        vertices_array = self.reader.project.elements[1].geometry.get_object().vertices

        # When
        vertices = self.reader.array_vertices(vertices_array)

        # Then
        expected_vertices = [[-1.0, -1.0, 0.0], [1.0, -1.0, 0.0], [1.0, 1.0, 0.0], [-1.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
        self.assertEqual(vertices, expected_vertices)

    def test_should_return_categories_attribute(self) -> None:
        # Given I get the attributes for my PointSet
        attributes = self.reader.project.elements[1].attributes

        # Then I should have three attributes
        self.assertEqual(len(attributes), 3)

        # And each attribute should be of the type Attribute
        for attribute in attributes:
            self.assertIsInstance(attribute, omf_python.Attribute)

        # And the first attribute should be named "Categories"
        attribute = attributes[0]
        self.assertEqual(attribute.name, "Categories")

        # And it should have a description
        self.assertEqual(attribute.description, "Divides the points into top and base.")

        # And its units should be whatever
        self.assertEqual(attribute.units, "whatever")

        # And it should have basic metadata
        metadata_string = attribute.metadata
        metadata = json.loads(metadata_string)
        expected_metadata = {
            "key": "value"
        }
        self.assertEqual(metadata, expected_metadata)

        # And the attribute should have a location
        self.assertEqual(attribute.location, "Vertices")

        # And the data it returns should be of type AttributeDataCategory
        self.assertIsInstance(attribute.get_data(), omf_python.AttributeDataCategory)

    def test_should_return_categories_attribute_data(self) -> None:
        # Given I get the attributes for my PointSet
        attributes = self.reader.project.elements[1].attributes

        # And I get the first Attribute
        attribute = attributes[0]

        # Then the data should be of type AttributeDataCategory
        self.assertIsInstance(attribute.get_data(), omf_python.AttributeDataCategory)

        # Then the category should contain: Indices and Names
        index_array = self.reader.project.elements[1].attributes[0].get_data().values
        self.assertIsInstance(index_array, omf_python.IndexArray)

        name_array = self.reader.project.elements[1].attributes[0].get_data().names
        self.assertIsInstance(name_array, omf_python.NameArray)
