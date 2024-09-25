import json
import omf_python
from os import path
from unittest import TestCase

class TestCategoryAttribute(TestCase):
    def setUp(self) -> None:
        omf_dir = path.join(path.dirname(__file__), "data")
        one_of_everything = path.join(omf_dir, "one_of_everything.omf")
        self.reader = omf_python.Reader(one_of_everything)
        self.project, _ = self.reader.project()

    def test_should_return_category_attribute_details(self) -> None:
        attributes = self.project.elements[1].attributes

        self.assertEqual(len(attributes), 3)

        for attribute in attributes:
            self.assertIsInstance(attribute, omf_python.Attribute)

        attribute = attributes[0]

        self.assertEqual(attribute.name, "Categories")

        self.assertEqual(attribute.description, "Divides the points into top and base.")

        self.assertEqual(attribute.units, "whatever")

        metadata_string = attribute.metadata
        metadata = json.loads(metadata_string)
        expected_metadata = {
            "key": "value"
        }
        self.assertEqual(metadata, expected_metadata)

        self.assertEqual(attribute.location, "Vertices")

    def test_should_return_category_attribute_array_instances(self) -> None:
        attributes = self.project.elements[1].attributes

        attribute_data = attributes[0].get_data()

        self.assertIsInstance(attribute_data, omf_python.AttributeDataCategory)

        self.assertIsInstance(attribute_data.values, omf_python.IndexArray)
        self.assertIsInstance(attribute_data.names, omf_python.NameArray)
        self.assertIsInstance(attribute_data.gradient, omf_python.GradientArray)
        self.assertIsInstance(attribute_data.attributes[0], omf_python.Attribute)

    def test_should_return_category_attribute_expected_values(self) -> None:
        attribute_data = self.project.elements[1].attributes[0].get_data()

        values = self.reader.array_indices(attribute_data.values)

        expected_values = [0, 0, 0, 0, 1]
        self.assertEqual(values, expected_values)

        names = self.reader.array_names(attribute_data.names)

        expected_names = ["Base", "Top"]
        self.assertEqual(names, expected_names)

        gradient = self.reader.array_gradient(attribute_data.gradient)

        expected_gradient = [[255, 128, 0, 255], [0, 128, 255, 255]]
        self.assertEqual(gradient, expected_gradient)

        category_attributes = attribute_data.attributes

        self.assertEqual(len(category_attributes), 1)
        self.assertIsInstance(category_attributes[0], omf_python.Attribute)
