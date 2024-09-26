import omf_python
from os import path
from unittest import TestCase

class TestPointSet(TestCase):
    def setUp(self) -> None:
        omf_dir = path.join(path.dirname(__file__), "data")
        one_of_everything = path.join(omf_dir, "one_of_everything.omf")
        self.reader = omf_python.Reader(one_of_everything)
        self.project, _ = self.reader.project()
        self.pointset = self.project.elements()[1]

    def test_should_return_expected_geometry_type(self) -> None:
        self.assertIsInstance(self.pointset.geometry(), omf_python.PointSet)

    def test_should_return_expected_origin(self) -> None:
        pointset_origin = self.pointset.geometry().origin
        self.assertEqual(pointset_origin, [0.0, 0.0, 0.0])

    def test_should_return_expected_vertices(self) -> None:
        # Given
        vertices_array = self.pointset.geometry().vertices

        # When
        vertices = self.reader.array_vertices(vertices_array)

        # Then
        expected_vertices = [[-1.0, -1.0, 0.0], [1.0, -1.0, 0.0], [1.0, 1.0, 0.0], [-1.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
        self.assertEqual(vertices, expected_vertices)
