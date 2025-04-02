from os import path
from unittest import TestCase

import numpy
import omf2


class TestCategoryAttribute(TestCase):
    def setUp(self) -> None:
        omf_dir = path.join(path.dirname(__file__), "data")
        one_of_everything = path.join(omf_dir, "one_of_everything.omf")
        self.reader = omf2.Reader(one_of_everything)
        self.project, _ = self.reader.project()
        self.attribute = self.project.elements()[1].attributes()[0]

    def test_should_return_category_attribute_details(self) -> None:
        self.assertIsInstance(self.attribute, omf2.Attribute)

        self.assertEqual(self.attribute.name, "Categories")
        self.assertEqual(
            self.attribute.description, "Divides the points into top and base."
        )
        self.assertEqual(self.attribute.units, "whatever")

        expected_metadata = {"key": "value"}
        self.assertDictEqual(self.attribute.metadata, expected_metadata)

        self.assertEqual(self.attribute.location, omf2.Location.Vertices)

    def test_should_return_category_attribute_array_instances(self) -> None:
        attribute_data = self.attribute.get_data()

        self.assertIsInstance(attribute_data, omf2.AttributeDataCategory)

        self.assertIsInstance(attribute_data.values, omf2.IndexArray)
        self.assertIsInstance(attribute_data.names, omf2.NameArray)
        self.assertIsInstance(attribute_data.gradient, omf2.GradientArray)
        self.assertIsInstance(attribute_data.attributes[0], omf2.Attribute)

    def test_should_return_category_attribute_expected_values(self) -> None:
        attribute_data = self.attribute.get_data()

        values, mask = self.reader.array_indices(attribute_data.values)

        expected_values = numpy.array([0, 0, 0, 0, 1])
        self.assertTrue(numpy.array_equal(values, expected_values))
        self.assertTrue(
            numpy.array_equal(
                mask, numpy.zeros(shape=len(expected_values), dtype=numpy.bool)
            )
        )

        names = self.reader.array_names(attribute_data.names)

        expected_names = ["Base", "Top"]
        self.assertEqual(names, expected_names)

        gradient = self.reader.array_gradient(attribute_data.gradient)

        expected_gradient = numpy.array([[255, 128, 0, 255], [0, 128, 255, 255]])
        self.assertTrue(numpy.array_equal(gradient, expected_gradient))

        category_attributes = attribute_data.attributes

        self.assertEqual(len(category_attributes), 1)
        self.assertIsInstance(category_attributes[0], omf2.Attribute)
