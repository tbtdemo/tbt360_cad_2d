"""Unit tests for the conservative IFC/OBJ GlobalId mapping exporter."""
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from export_ifc_obj_mapping import extract

GUID = "3h7B1JG5D4QQ4W8gcZQ0Xa"
REL = "1A2b3C4d5E6f7G8h9I0j1K"
STOREY = "2A2b3C4d5E6f7G8h9I0j1K"

class ExportMappingTests(unittest.TestCase):
    def test_ifc_storey_and_unique_obj_group_guid(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            ifc = root / "sample.ifc"
            obj = root / "sample.obj"
            ifc.write_text(
                "ISO-10303-21;\nDATA;\n"
                f"#1=IFCBEAM('{GUID}',$,'B1',$,$,$,$,$);\n"
                f"#2=IFCBUILDINGSTOREY('{STOREY}',$,'Tầng 2',$,$,$,$,$,$);\n"
                f"#3=IFCRELCONTAINEDINSPATIALSTRUCTURE('{REL}',$,$,$,(#1),#2);\n"
                "ENDSEC;\nEND-ISO-10303-21;\n", encoding="utf-8"
            )
            obj.write_text(f"o Beam_{GUID}\nf 1 2 3\no NotMapped\n", encoding="utf-8")
            catalog, unmatched = extract(ifc, obj, "Kết cấu")
            self.assertEqual(len(catalog["objects"]), 1)
            item = catalog["objects"][0]
            self.assertEqual(item["global_id"], GUID)
            self.assertEqual(item["storey"], "Tầng 2")
            self.assertEqual(item["discipline"], "Kết cấu")
            self.assertEqual(unmatched, ["NotMapped"])

    def test_group_guid_with_underscore_prefix_is_mapped(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            ifc = root / "sample.ifc"
            obj = root / "sample.obj"
            ifc.write_text("ISO-10303-21;\nDATA;\n" + f"#1=IFCBEAM('{GUID}',$,'B1',$,$,$,$,$);\n" + "ENDSEC;\n", encoding="utf-8")
            obj.write_text(f"g Beam_{GUID}\n", encoding="utf-8")
            catalog, unmatched = extract(ifc, obj, "Kết cấu")
            self.assertEqual(len(catalog["objects"]), 1)
            self.assertEqual(catalog["objects"][0]["global_id"], GUID)
            self.assertEqual(unmatched, [])

    def test_unmatched_groups_are_not_guessed(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            ifc = root / "sample.ifc"
            obj = root / "sample.obj"
            ifc.write_text("ISO-10303-21;\nDATA;\nENDSEC;\n", encoding="utf-8")
            obj.write_text("g Wall001\n", encoding="utf-8")
            catalog, unmatched = extract(ifc, obj, "")
            self.assertEqual(catalog["objects"], [])
            self.assertEqual(unmatched, ["Wall001"])

if __name__ == "__main__":
    unittest.main()
