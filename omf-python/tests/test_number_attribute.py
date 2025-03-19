import datetime
from os import path
from unittest import TestCase

import numpy
import omf2
from omf2 import BoundaryType


class TestNumberAttribute(TestCase):
    def setUp(self) -> None:
        omf_dir = path.join(path.dirname(__file__), "data")
        one_of_everything = path.join(omf_dir, "one_of_everything.omf")
        continuous_colormap = path.join(omf_dir, "continuous_colormap.omf")

        self.reader = omf2.Reader(one_of_everything)
        self.project, _ = self.reader.project()
        self.attribute = self.project.elements()[0].attributes()[2]

        self.ccmap_reader = omf2.Reader(continuous_colormap)
        self.ccmap_project, _ = self.ccmap_reader.project()

    def test_should_return_number_attribute_instance(self) -> None:
        self.assertIsInstance(self.attribute.get_data(), omf2.AttributeDataNumber)

    def test_should_return_number_attribute_values(self) -> None:
        attribute_data = self.attribute.get_data()

        expected_values = numpy.array(
            [
                "2000-01-01T00:00:00.000000",
                "2000-01-01T01:00:00.000000",
                "2000-01-01T02:00:00.000000",
                "2000-01-01T03:00:00.000000",
                "2000-01-01T04:00:00.000000",
            ],
            dtype="datetime64[us]",
        )
        values, mask = self.reader.array_numbers(attribute_data.values)

        self.assertTrue(numpy.array_equal(expected_values, values))
        self.assertTrue(
            numpy.array_equal(
                numpy.zeros(shape=len(expected_values), dtype=numpy.bool), mask
            )
        )

    def test_should_handle_empty_colormap(self) -> None:
        number_attribute_data = self.project.elements()[0].attributes()[0].get_data()

        self.assertEqual(number_attribute_data.colormap, None)

    def test_should_return_discrete_colormap(self) -> None:
        colormap = self.attribute.get_data().colormap

        self.assertIsInstance(colormap, omf2.NumberColormapDiscrete)
        self.assertIsInstance(colormap.boundaries, omf2.BoundaryArray)
        self.assertIsInstance(colormap.gradient, omf2.GradientArray)

    def test_should_return_discrete_colormap_boundaries(self) -> None:
        boundary_array = self.attribute.get_data().colormap.boundaries
        actual_boundaries = self.reader.array_boundaries(boundary_array)

        expected_boundaries = [
            (
                datetime.datetime(2000, 1, 1, 1, 0, tzinfo=datetime.timezone.utc),
                BoundaryType.Less,
            ),
            (
                datetime.datetime(2000, 1, 1, 2, 0, tzinfo=datetime.timezone.utc),
                BoundaryType.Less,
            ),
            (
                datetime.datetime(2000, 1, 1, 3, 0, tzinfo=datetime.timezone.utc),
                BoundaryType.LessEqual,
            ),
        ]
        self.assertEqual(actual_boundaries, expected_boundaries)

    def test_should_return_discrete_colormap_gradient(self) -> None:
        gradient_array = self.attribute.get_data().colormap.gradient
        actual_gradient = self.reader.array_gradient(gradient_array)

        expected_gradient = numpy.array(
            [
                [0, 0, 255, 255],
                [0, 255, 0, 255],
                [255, 0, 0, 255],
                [255, 255, 255, 255],
            ]
        )
        self.assertTrue(numpy.array_equal(actual_gradient, expected_gradient))

    def test_should_return_continuous_colormap(self) -> None:
        colormap = self.ccmap_project.elements()[0].attributes()[0].get_data().colormap

        self.assertIsInstance(colormap, omf2.NumberColormapContinuous)
        self.assertIsInstance(colormap.gradient, omf2.GradientArray)

    def test_should_return_continuous_colormap_float_range(self) -> None:
        min, max = (
            self.ccmap_project.elements()[0].attributes()[0].get_data().colormap.range()
        )

        self.assertEqual(min, 0.0)
        self.assertEqual(max, 2.0)

    def test_should_return_continuous_colormap_date_range(self) -> None:
        min, max = (
            self.ccmap_project.elements()[0].attributes()[1].get_data().colormap.range()
        )

        expected_min = datetime.date(1995, 5, 1)
        expected_max = datetime.date(1998, 8, 1)

        self.assertEqual(min, expected_min)
        self.assertEqual(max, expected_max)

    def test_should_return_continuous_colormap_datetime_range(self) -> None:
        min, max = (
            self.ccmap_project.elements()[0].attributes()[2].get_data().colormap.range()
        )

        expected_min = datetime.datetime(1995, 5, 1, 5, 1, tzinfo=datetime.timezone.utc)
        expected_max = datetime.datetime(1998, 8, 1, 8, 1, tzinfo=datetime.timezone.utc)

        self.assertEqual(min, expected_min)
        self.assertEqual(max, expected_max)

    def test_should_return_continuous_colormap_f32_range(self) -> None:
        min, max = (
            self.ccmap_project.elements()[0].attributes()[3].get_data().colormap.range()
        )

        self.assertEqual(min, 0.0)
        self.assertEqual(max, 2.0)

    def test_should_return_continuous_colormap_i64_range(self) -> None:
        min, max = (
            self.ccmap_project.elements()[0].attributes()[4].get_data().colormap.range()
        )

        self.assertEqual(min, 0)
        self.assertEqual(max, 200)
