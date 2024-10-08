import omf_python
from os import path
from unittest import TestCase


class TestGridSurface(TestCase):
    def setUp(self) -> None:
        self.omf_dir = path.join(path.dirname(__file__), "data")
        self.one_of_everything = path.join(self.omf_dir, "one_of_everything.omf")

    def test_should_contain_grid_surface_geometry(self) -> None:
        reader = omf_python.Reader(self.one_of_everything)
        project, _ = reader.project()
        grid_surface = project.elements()[3].geometry()
        self.assertIsInstance(grid_surface, omf_python.GridSurface)

        orientation = grid_surface.orient

        self.assertEqual([-1.5, -1.5, 0], orientation.origin)
        self.assertEqual([1, 0, 0], orientation.u)
        self.assertEqual([0, 1, 0], orientation.v)

        height_array = grid_surface.heights()
        self.assertEqual(9, height_array.item_count())
        self.assertEqual(
            [0, 0, 0, 0, 2, 0, 0, 0, 0], reader.array_scalars(height_array)
        )

        grid = grid_surface.grid
        self.assertIsInstance(grid, omf_python.Tensor)
        self.assertEqual([2, 2], grid.count())

        u = grid.u
        self.assertIsInstance(u, omf_python.ScalarArray)
        self.assertEqual(2, u.item_count())
        self.assertEqual([1, 2], reader.array_scalars(u))

        v = grid.v
        self.assertIsInstance(v, omf_python.ScalarArray)
        self.assertEqual(2, v.item_count())
        self.assertEqual([1.5, 1.5], reader.array_scalars(v))
