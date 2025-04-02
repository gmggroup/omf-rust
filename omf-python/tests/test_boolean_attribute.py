from os import path
from unittest import TestCase

import numpy
import omf2


class TestBooleanAttribute(TestCase):
    def setUp(self) -> None:
        omf_dir = path.join(path.dirname(__file__), "data")
        one_of_everything = path.join(omf_dir, "one_of_everything.omf")
        self.reader = omf2.Reader(one_of_everything)
        self.project, _ = self.reader.project()

        # Get the "Regular block model" element, and its "Filter" attribute.
        self.attribute = self.project.elements()[4].attributes()[0]

    def test_should_return_boolean_attribute_instance(self) -> None:
        self.assertIsInstance(self.attribute.get_data(), omf2.AttributeDataBoolean)

    def test_should_return_boolean_attribute_values_item_count(self) -> None:
        actual_count = self.attribute.get_data().values.item_count()
        self.assertEqual(actual_count, 8)

    def test_should_return_boolean_attribute_values(self) -> None:
        attribute_data = self.attribute.get_data()

        expected_values = numpy.array(
            [False, False, False, False, False, False, False, True]
        )
        values, mask = self.reader.array_booleans(attribute_data.values)

        self.assertTrue(numpy.array_equal(expected_values, values))
        self.assertTrue(
            numpy.array_equal(
                numpy.zeros(shape=len(expected_values), dtype=numpy.bool), mask
            )
        )
