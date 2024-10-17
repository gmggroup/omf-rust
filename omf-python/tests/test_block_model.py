import omf_python
from os import path
from unittest import TestCase


class TestBlockModel(TestCase):
    def setUp(self) -> None:
        self.omf_dir = path.join(path.dirname(__file__), "data")
        self.one_of_everything = path.join(self.omf_dir, "one_of_everything.omf")

    def test_should_return_expected_regular_block_model(self) -> None:
        reader = omf_python.Reader(self.one_of_everything)
        project, _ = reader.project()
        block_model : omf_python.BlockModel = project.elements()[4].geometry()
        self.assertIsInstance(block_model, omf_python.BlockModel)

        orientation = block_model.orient
        self.assertEqual([-1, -1, -1], orientation.origin)
        self.assertEqual([1, 0, 0], orientation.u)
        self.assertEqual([0, 1, 0], orientation.v)
        self.assertEqual([0, 0, 1], orientation.w)

        grid = block_model.grid
        self.assertIsInstance(grid, omf_python.Grid3Regular)
        self.assertEqual([1,1,1], grid.size)
        self.assertEqual([2,2,2], grid.count())
        self.assertEqual(8, grid.flat_count())
        self.assertEqual(27, grid.flat_corner_count())

        subblocks = block_model.subblocks
        self.assertIsNone(subblocks)

    def test_should_have_expected_tensor_block_model(self) -> None:
        reader = omf_python.Reader(self.one_of_everything)
        project, _ = reader.project()
        block_model : omf_python.BlockModel = project.elements()[5].geometry()
        self.assertIsInstance(block_model, omf_python.BlockModel)

        grid = block_model.grid
        self.assertIsInstance(grid, omf_python.Grid3Tensor)
        self.assertEqual([2,2,2], grid.count())
        self.assertEqual(8, grid.flat_count())
        self.assertEqual(27, grid.flat_corner_count())

        u = grid.u
        self.assertIsInstance(u, omf_python.ScalarArray)
        self.assertEqual(2, u.item_count())
        self.assertEqual([0.6666, 1.333], reader.array_scalars(u))

        v = grid.v
        self.assertIsInstance(v, omf_python.ScalarArray)
        self.assertEqual(2, v.item_count())
        self.assertEqual([0.6666, 1.333], reader.array_scalars(v))

        w = grid.w
        self.assertIsInstance(w, omf_python.ScalarArray)
        self.assertEqual(2, w.item_count())
        self.assertEqual([1.0, 1.0], reader.array_scalars(w))

    def test_should_have_expected_regular_subblocks(self) -> None:
        reader = omf_python.Reader(self.one_of_everything)
        project, _ = reader.project()
        block_model : omf_python.BlockModel = project.elements()[6].geometry()
        self.assertIsInstance(block_model, omf_python.BlockModel)

        subblocks_array = block_model.subblocks
        self.assertIsInstance(subblocks_array, omf_python.RegularSubblockArray)

        expected_regular_subblock_values = [
            ([0, 0, 0], [0, 0, 0, 4, 4, 4]),
            ([1, 0, 0], [0, 0, 0, 4, 4, 4]),
            ([0, 1, 0], [0, 0, 0, 4, 4, 4]),
            ([1, 1, 0], [0, 0, 0, 4, 4, 4]),
            ([0, 0, 1], [0, 0, 0, 4, 4, 4]),
            ([1, 0, 1], [0, 0, 0, 4, 4, 4]),
            ([0, 1, 1], [0, 0, 0, 4, 4, 4]),
            ([1, 1, 1], [0, 0, 0, 2, 2, 2]),
            ([1, 1, 1], [0, 2, 0, 1, 3, 1]),
            ([1, 1, 1], [2, 0, 0, 3, 1, 1]),
            ([1, 1, 1], [0, 0, 2, 1, 1, 3]),
        ]
        self.assertListEqual(
            expected_regular_subblock_values,
            reader.array_regular_subblocks(subblocks_array)
        )

    def test_should_have_expected_freeform_subblocks(self) -> None:
        reader = omf_python.Reader(self.one_of_everything)
        project, _ = reader.project()
        block_model : omf_python.BlockModel = project.elements()[7].geometry()
        self.assertIsInstance(block_model, omf_python.BlockModel)

        subblocks_array = block_model.subblocks
        self.assertIsInstance(subblocks_array, omf_python.FreeformSubblockArray)

        expected_freeform_subblock_values = [
            ([0, 0, 0], [0.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
            ([1, 0, 0], [0.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
            ([0, 1, 0], [0.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
            ([1, 1, 0], [0.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
            ([0, 0, 1], [0.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
            ([1, 0, 1], [0.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
            ([0, 1, 1], [0.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
            ([1, 1, 1], [0.0, 0.0, 0.0, 1.0, 1.0, 0.3333]),
            ([1, 1, 1], [0.0, 0.0, 0.3333, 0.75, 0.75, 0.6666]),
            ([1, 1, 1], [0.0, 0.0, 0.6666, 0.5, 0.5, 1.0]),
        ]

        freeform_subblock_values = reader.array_freeform_subblocks(subblocks_array)

        # Round floating point values to check they're almost equal
        freeform_subblock_values = [
            (a, [round(i, 4) for i in b]) for (a, b) in freeform_subblock_values
        ]
        self.assertListEqual(expected_freeform_subblock_values, freeform_subblock_values)
