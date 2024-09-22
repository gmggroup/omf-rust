import json
import omf_python
from os import path
from unittest import TestCase

class TestCategoryAttribute(TestCase):
    def setUp(self) -> None:
        omf_dir = path.join(path.dirname(__file__), "data")
        one_of_everything = path.join(omf_dir, "one_of_everything.omf")
        self.reader = omf_python.Reader(one_of_everything)

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

    def test_should_return_categories_attribute_data(self) -> None:
        # Given I get the attributes for my PointSet
        attributes = self.reader.project.elements[1].attributes

        # And I get the data from the first Attribute
        attribute_data = attributes[0].get_data()

        # Then the data should be of type AttributeDataCategory
        self.assertIsInstance(attribute_data, omf_python.AttributeDataCategory)

        # Then the attribute data should contain: Values, Names, Gradient and Attributes
        self.assertIsInstance(attribute_data.values, omf_python.IndexArray)
        self.assertIsInstance(attribute_data.names, omf_python.NameArray)
        self.assertIsInstance(attribute_data.gradient, omf_python.GradientArray)
        self.assertIsInstance(attribute_data.attributes[0], omf_python.Attribute)

    def test_should_return_category_values(self) -> None:
        # Given I have the category attribute data
        attribute_data = self.reader.project.elements[1].attributes[0].get_data()

        # When I read the values for the category
        values = self.reader.array_indices(attribute_data.values)

        # Then I should get the expected results
        expected_values = [0, 0, 0, 0, 1]
        self.assertEqual(values, expected_values)

        # And when I read the names
        names = self.reader.array_names(attribute_data.names)

        # Then I should get the expected names back
        expected_names = ["Base", "Top"]
        self.assertEqual(names, expected_names)

        # And when I read the gradient
        gradient = self.reader.array_gradient(attribute_data.gradient)

        expected_gradient = [[255, 128, 0, 255], [0, 128, 255, 255]]
        self.assertEqual(gradient, expected_gradient)

        # And when I get the category sub-attributes
        category_attributes = attribute_data.attributes

        # There should be one attribute
        self.assertEqual(len(category_attributes), 1)
        self.assertIsInstance(category_attributes[0], omf_python.Attribute)
