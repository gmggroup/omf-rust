import omf_python
from os import path
from unittest import TestCase


class TestColorAttribute(TestCase):
    def setUp(self) -> None:
        omf_dir = path.join(path.dirname(__file__), "data")
        one_of_everything = path.join(omf_dir, "one_of_everything.omf")
        self.reader = omf_python.Reader(one_of_everything)
        self.project, _ = self.reader.project()
        self.attribute = self.project.elements()[0].attributes()[1]

    def test_should_return_expected_color_attributes(self) -> None:
        values = self.attribute.get_data().values
        self.assertEqual(6, values.item_count())

        color_array = self.reader.array_color(values)
        self.assertEqual(6, len(color_array))
        expected_colors = [
            [255, 0, 0, 255],
            [255, 255, 0, 255],
            [0, 255, 0, 255],
            [0, 0, 255, 255],
            [255, 255, 255, 255],
            [255, 255, 255, 255],
        ]
        self.assertEqual(expected_colors, color_array)
