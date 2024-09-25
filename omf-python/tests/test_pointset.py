import omf_python
from os import path
from unittest import TestCase

class TestPointSet(TestCase):
    def setUp(self) -> None:
        omf_dir = path.join(path.dirname(__file__), "data")
        one_of_everything = path.join(omf_dir, "one_of_everything.omf")
        self.reader = omf_python.Reader(one_of_everything)
        self.project, _ = self.reader.project()

    def test_should_return_expected_geometry_type(self) -> None:
        pointset_type = self.project.elements[1].geometry.type_name()

        self.assertEqual(pointset_type, "PointSet")

    def test_should_return_expected_origin(self) -> None:
        pointset_origin = self.project.elements[1].geometry.get_object().origin

        self.assertEqual(pointset_origin, [0.0, 0.0, 0.0])

    def test_should_return_expected_vertices(self) -> None:
        # Given
        vertices_array = self.project.elements[1].geometry.get_object().vertices

        # When
        vertices = self.reader.array_vertices(vertices_array)

        # Then
        expected_vertices = [[-1.0, -1.0, 0.0], [1.0, -1.0, 0.0], [1.0, 1.0, 0.0], [-1.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
        self.assertEqual(vertices, expected_vertices)
