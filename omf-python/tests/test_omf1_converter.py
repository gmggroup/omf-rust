import omf_python
from os import path
from unittest import TestCase
from tempfile import NamedTemporaryFile


class TestOmf1Converter(TestCase):
    def setUp(self) -> None:
        self.test_data_dir = path.join(path.dirname(__file__), "../../tests")

    def test_should_convert_omf1_to_omf2_file(self) -> None:
        # Given
        omf1_file = path.join(self.test_data_dir, "omf1/test_proj.omf")
        self.assertTrue(omf_python.detect_omf1(omf1_file))

        converter = omf_python.Omf1Converter()

        with NamedTemporaryFile(suffix=".omf") as omf2_file:
            # When
            converter.convert(omf1_file, omf2_file.name)

            # Then
            reader = omf_python.Reader(omf2_file.name)
            project, _ = reader.project()
            self.assertEqual(len(project.elements()), 2)

    def test_should_return_expected_problems(self) -> None:
        # Given
        omf1_file = path.join(self.test_data_dir, "omf1/test_proj.omf")
        self.assertTrue(omf_python.detect_omf1(omf1_file))

        converter = omf_python.Omf1Converter()

        # When
        with NamedTemporaryFile(suffix=".omf") as omf2_file:
            problems = converter.convert(omf1_file, omf2_file.name)

        # Then
        self.assertEqual(len(problems), 1)

        problem = problems[0]

        self.assertEqual(str(problem), "Warning: 'Project::elements[..]::name' contains duplicate of \"\", inside ''")
        self.assertEqual(problem.name, "")
        self.assertEqual(problem.field, "elements[..]::name")
        self.assertEqual(problem.reason, "contains duplicate of \"\"")
        self.assertEqual(problem.type_name, "Project")
        self.assertEqual(problem.is_error(), False)

    def test_should_raise_expected_exception_if_file_not_found(self) -> None:
        # Given
        omf1_file = path.join(self.test_data_dir, "omf1/testfilenotfound.omf")

        converter = omf_python.Omf1Converter()

        with NamedTemporaryFile(suffix=".omf") as omf2_file:
            # When
            with self.assertRaises(omf_python.OmfFileIoException) as context:
                converter.convert(omf1_file, omf2_file.name)

            # Then
            self.assertEqual(str(context.exception), "File IO error: No such file or directory (os error 2)")

    def test_should_return_expected_default_limits(self) -> None:
        # Given
        converter = omf_python.Omf1Converter()

        # When
        limits = converter.limits()

        # Then
        self.assertEqual(limits.json_bytes, 1024 * 1024)
        self.assertEqual(limits.image_bytes, 16 * 1024 * 1024 * 1024)
        self.assertEqual(limits.image_dim, None)
        self.assertEqual(limits.validation, 100)

    def test_should_set_limits(self) -> None:
        # Given
        converter = omf_python.Omf1Converter()

        limits = omf_python.Limits()
        limits.json_bytes = 1
        limits.image_bytes = 2
        limits.image_dim = 3
        limits.validation = 4

        # When
        converter.set_limits(limits)

        # Then
        updated_limits = converter.limits()
        self.assertEqual(updated_limits.json_bytes, limits.json_bytes)
        self.assertEqual(updated_limits.image_bytes, limits.image_bytes)
        self.assertEqual(updated_limits.image_dim, limits.image_dim)
        self.assertEqual(updated_limits.validation, limits.validation)

    def test_should_raise_exception_if_limit_invalid(self) -> None:
        limits = omf_python.Limits()
        with self.assertRaises(OverflowError):
            limits.json_bytes = -1

    def test_should_raise_exception_if_json_bytes_limit_reached(self) -> None:
        # Given
        omf1_file = path.join(self.test_data_dir, "omf1/test_proj.omf")
        self.assertTrue(omf_python.detect_omf1(omf1_file))

        converter = omf_python.Omf1Converter()
        limits = converter.limits()
        limits.json_bytes = 0

        with NamedTemporaryFile(suffix=".omf") as omf2_file:
            # When
            converter.set_limits(limits)
            with self.assertRaises(omf_python.OmfLimitExceededException) as context:
                converter.convert(omf1_file, omf2_file.name)

            # Then
            self.assertEqual(str(context.exception), "Error: safety limit exceeded")

    def test_should_get_compression(self) -> None:
        # Given
        converter = omf_python.Omf1Converter()

        # When
        compression = converter.compression()

        # Then
        self.assertEqual(compression, 6)

    def test_should_set_compression(self) -> None:
        # Given
        converter = omf_python.Omf1Converter()

        # When
        converter.set_compression(9)

        # Then
        self.assertEqual(converter.compression(), 9)

    def test_should_limit_max_compression(self) -> None:
        # Given
        converter = omf_python.Omf1Converter()

        # When
        converter.set_compression(10)

        # Then
        self.assertEqual(converter.compression(), 9)

    def test_high_compression_reduces_file_size(self) -> None:
        # Given
        omf1_file = path.join(self.test_data_dir, "omf1/test_proj.omf")
        converter = omf_python.Omf1Converter()
        converter.set_compression(0)

        # When
        with NamedTemporaryFile(suffix=".omf") as omf2_file:
            converter.convert(omf1_file, omf2_file.name)
            omf_python.Reader(omf2_file.name)
            low_compression_file_size = path.getsize(omf2_file.name)

        converter.set_compression(9)

        with NamedTemporaryFile(suffix=".omf") as omf2_file:
            converter.convert(omf1_file, omf2_file.name)
            omf_python.Reader(omf2_file.name)
            high_compression_file_size = path.getsize(omf2_file.name)

        # Then
        self.assertLess(high_compression_file_size, low_compression_file_size)
