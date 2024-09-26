import datetime
import omf_python
from os import path
from unittest import TestCase

class TestNumberAttribute(TestCase):
    def setUp(self) -> None:
        omf_dir = path.join(path.dirname(__file__), "data")
        one_of_everything = path.join(omf_dir, "one_of_everything.omf")
        continuous_colormap = path.join(omf_dir, "continuous_colormap.omf")

        self.reader = omf_python.Reader(one_of_everything)
        self.project, _ = self.reader.project()
        self.attribute = self.project.elements()[0].attributes()[2]
        self.ccmap_reader = omf_python.Reader(continuous_colormap)

    def test_should_return_number_attribute_instance(self) -> None:
        self.assertIsInstance(self.attribute.get_data(), omf_python.AttributeDataNumber)

    def test_should_return_number_attribute_values(self) -> None:
        attribute_data = self.attribute.get_data()

        expected_values = [946684800.0, 946688400.0, 946692000.0, 946695600.0, 946699200.0]
        actual_values = self.reader.array_numbers(attribute_data.values)

        self.assertEqual(expected_values, actual_values)

    def test_should_handle_empty_colormap(self) -> None:
        number_attribute_data = self.reader.project.elements[0].attributes[0].get_data()

        self.assertEqual(number_attribute_data.colormap, None)

    def test_should_return_discrete_colormap(self) -> None:
        colormap = self.reader.project.elements[0].attributes[2].get_data().colormap

        self.assertIsInstance(colormap, omf_python.NumberColormapDiscrete)
        self.assertIsInstance(colormap.boundaries, omf_python.BoundaryArray)
        self.assertIsInstance(colormap.gradient, omf_python.GradientArray)

    def test_should_return_discrete_colormap_boundaries(self) -> None:
        boundary_array = self.reader.project.elements[0].attributes[2].get_data().colormap.boundaries
        actual_boundaries = self.reader.array_boundaries(boundary_array)

        expected_boundaries = [946688400.0, 946692000.0, 946695600.0]
        self.assertEqual(actual_boundaries, expected_boundaries)

    def test_should_return_discrete_colormap_gradient(self) -> None:
        gradient_array = self.reader.project.elements[0].attributes[2].get_data().colormap.gradient
        actual_gradient = self.reader.array_gradient(gradient_array)

        expected_gradient = [[0, 0, 255, 255], [0, 255, 0, 255], [255, 0, 0, 255], [255, 255, 255, 255]]
        self.assertEqual(actual_gradient, expected_gradient)

    def test_should_return_continuous_colormap(self) -> None:
        colormap = self.ccmap_reader.project.elements[0].attributes[0].get_data().colormap

        self.assertIsInstance(colormap, omf_python.NumberColormapContinuous)
        self.assertIsInstance(colormap.range, omf_python.NumberRangeFloat)
        self.assertIsInstance(colormap.gradient, omf_python.GradientArray)

    def test_should_return_continuous_colormap_float_range(self) -> None:
        range = self.ccmap_reader.project.elements[0].attributes[0].get_data().colormap.range

        self.assertEqual(range.min, 0.0)
        self.assertEqual(range.max, 2.0)

    def test_should_return_continuous_colormap_date_range(self) -> None:
        range = self.ccmap_reader.project.elements[0].attributes[1].get_data().colormap.range

        self.assertIsInstance(range, omf_python.NumberRangeDate)

        expected_min = datetime.date(1995, 5, 1)
        expected_max = datetime.date(1998, 8, 1)

        self.assertEqual(range.min, expected_min)
        self.assertEqual(range.max, expected_max)

    def test_should_return_continuous_colormap_datetime_range(self) -> None:
        range = self.ccmap_reader.project.elements[0].attributes[2].get_data().colormap.range

        self.assertIsInstance(range, omf_python.NumberRangeDateTime)

        expected_min = datetime.datetime(1995, 5, 1, 5, 1, tzinfo=datetime.timezone.utc)
        expected_max = datetime.datetime(1998, 8, 1, 8, 1, tzinfo=datetime.timezone.utc)

        self.assertEqual(range.min, expected_min)
        self.assertEqual(range.max, expected_max)
