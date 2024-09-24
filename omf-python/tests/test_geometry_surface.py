import omf_python
from os import path
from unittest import TestCase


class TestGeometrySurface(TestCase):
    def setUp(self) -> None:
        self.examples_dir = path.join(path.dirname(__file__), "../../examples")

    def test_should_contain_surface_geometery(self) -> None:
        # Given the pyramid sample omf file
        omf_file = path.join(self.examples_dir, "pyramid/pyramid.omf")

        # When
        reader = omf_python.Reader(omf_file)
        project, _ = reader.project()

        # Then there is one surface element
        surfaces = [
            s for s in project.elements if s.geometry.type_name() == "Surface"
        ]
        self.assertEqual(1, len(surfaces))

        # And it has the correct type of omf_python.Surface
        surface = surfaces[0].geometry.get_object()
        self.assertIsInstance(surface, omf_python.Surface)

        # And it contains 6 triangles
        triangles_array = surface.triangles
        self.assertEqual(6, triangles_array.item_count)

        # And it contains 5 vertices
        vertices_array = surface.vertices
        self.assertEqual(5, vertices_array.item_count)

        # And the vertices are what we expect
        vertices = reader.array_vertices(vertices_array)
        VERTICES = [[-1, -1, 0], [1, -1, 0], [1, 1, 0], [-1, 1, 0], [0, 0, 1]]
        self.assertEqual(VERTICES, vertices)

        # And the triangles are what we expect
        triangles = reader.array_triangles(triangles_array)
        TRIANGLES = [
            [0, 1, 4],
            [1, 2, 4],
            [2, 3, 4],
            [3, 0, 4],
            [0, 2, 1],
            [0, 3, 2],
        ]
        self.assertEqual(TRIANGLES, triangles)

    def test_should_contain_color(self) -> None:
        # Given the pyramid sample omf file
        omf_file = path.join(self.examples_dir, "pyramid/pyramid.omf")

        # When
        reader = omf_python.Reader(omf_file)
        project, _ = reader.project()

        # Then there is one surface element
        surfaces = [
            s for s in project.elements if s.geometry.type_name() == "Surface"
        ]
        self.assertEqual(1, len(surfaces))

        # And it has the correct type of omf_python.Surface
        surface = surfaces[0].geometry.get_object()
        self.assertIsInstance(surface, omf_python.Surface)

        # And it has the correct color
        color = surfaces[0].color
        self.assertEqual(255, color.red)
        self.assertEqual(255, color.green)
        self.assertEqual(0, color.blue)
        self.assertEqual(255, color.alpha)
