from pathlib import Path
import random
import uuid
import datetime
# This file should be fixed so as to generate the entire dev-test_db.sql file and be a little more robust and modular.

def random_datetime(min_year: int, max_year: int) -> datetime.datetime:
    # generate a datetime in format yyyy-mm-dd hh:mm:ss.000000
    start = datetime.datetime(min_year, 1, 1, 00, 00, 00)
    n_years = max_year - min_year + 1
    end = start + datetime.timedelta(days=365 * n_years)

    return start + ((end - start) * random.random())


insert_sample_metadata = "insert into sample_metadata (id, name, submitted_by, lab_id, received_at, species, tissue) values"
insert_specimen = "insert into specimen (id, legacy_id, metadata_id, type, embedded_in, preserved_with) values"
people_ids = (
    "0192124b-4895-704a-8162-4b5b9cc9408c",
    "03bf73a7-9298-4651-8237-f401a6a824a2",
)
lab_ids = (
    "0192124b-f34a-776f-b82a-bad6e854c4e1",
    "e12c210b-6368-4746-a460-a85e1cc64148",
)

for i in range(1000):
    metadata_id = uuid.uuid4()
    name = f"sample-{i}"
    submitted_by_id = random.choice(people_ids)
    lab_id = random.choice(lab_ids)
    received_at = random_datetime(1999, 2025)
    species = random.choice(("homo_sapiens", "mus_musculus"))
    species = "{" + '"' + species + '"' + "}"
    tissue = "kidney"

    values = [metadata_id, name, submitted_by_id, lab_id, received_at, species, tissue]  # type: ignore
    values = [f"'{v}'" for v in values]  # type: ignore
    values = "(" + ",".join(values) + ")"

    if i == 0:
        insert_sample_metadata = f"{insert_sample_metadata} {values}"
    else:
        insert_sample_metadata = f"{insert_sample_metadata}, {values}"

    specimen_id = uuid.uuid4()
    legacy_id = f"id-{i}"
    type_ = "block"
    embedded_in = "paraffin"
    preserved_with = "formaldehyde_derivative_fixation"

    values = [specimen_id, legacy_id, metadata_id, type_, embedded_in, preserved_with]  # type: ignore
    values = [f"'{v}'" for v in values]  # type: ignore
    values = "(" + ",".join(values) + ")"

    if i == 0:
        insert_specimen = f"{insert_specimen} {values}"
    else:
        insert_specimen = f"{insert_specimen}, {values}"

statement = f"{insert_sample_metadata};\n\n{insert_specimen};"

Path("dev-test_db.sql").write_text(statement)
