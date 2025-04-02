from os import path
from unittest import TestCase

import numpy
import omf2


class TestVectorAttribute(TestCase):
    def setUp(self) -> None:
        omf_dir = path.join(path.dirname(__file__), "data")
        one_of_everything = path.join(omf_dir, "one_of_everything.omf")
        self.reader = omf2.Reader(one_of_everything)
        self.project, _ = self.reader.project()
        self.attribute = self.project.elements()[1].attributes()[1]

    def test_should_return_vector_attribute_instance(self) -> None:
        self.assertIsInstance(self.attribute.get_data(), omf2.AttributeDataVector)

    def test_should_return_vector_attribute_values(self) -> None:
        attribute_data = self.attribute.get_data()

        expected_values = numpy.array(
            [
                [1.0, 0.0],
                [1.0, 1.0],
                [0.0, 1.0],
                [0.0, 0.0],
                [0.0, 0.0],
            ]
        )
        expected_mask = numpy.array(
            [
                False,
                False,
                False,
                False,
                True,
            ]
        )
        values, mask = self.reader.array_vectors(attribute_data.values)

        self.assertEqual(numpy.float32, values.dtype)
        self.assertTrue(numpy.array_equal(expected_values, values))
        self.assertTrue(numpy.array_equal(expected_mask, mask))
