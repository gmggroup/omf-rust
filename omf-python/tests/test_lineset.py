import omf_python
from os import path
from unittest import TestCase


class TestLineSet(TestCase):
    def setUp(self) -> None:
        omf_dir = path.join(path.dirname(__file__), "data")
        one_of_everything = path.join(omf_dir, "one_of_everything.omf")
        self.reader = omf_python.Reader(one_of_everything)
        self.project, _ = self.reader.project()

    def test_should_return_expected_geometry_type(self) -> None:
        lineset_type = self.project.elements[2].geometry.type_name()

        self.assertEqual(lineset_type, "LineSet")

    def test_should_return_expected_origin(self) -> None:
        lineset_origin = self.project.elements[2].geometry.get_object().origin

        self.assertEqual(lineset_origin, [0.0, 0.0, 0.0])

    def test_should_return_expected_vertices(self) -> None:
        # Given
        vertices_array = self.project.elements[2].geometry.get_object().vertices

        # When
        vertices = self.reader.array_vertices(vertices_array)

        # Then
        expected_vertices = [
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [1.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ]
        self.assertEqual(vertices, expected_vertices)

    def test_should_return_expected_segments(self) -> None:
        # Given
        segments_array = self.project.elements[2].geometry.get_object().segments

        # When
        segments = self.reader.array_segments(segments_array)

        # Then
        expected_segments = [
            [0, 1],
            [1, 2],
            [2, 3],
            [3, 0],
            [0, 4],
            [1, 4],
            [2, 4],
            [3, 4],
        ]
        self.assertEqual(segments, expected_segments)
