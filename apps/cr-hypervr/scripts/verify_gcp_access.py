#!/usr/bin/env python3
"""
Lightweight GCP access verifier used by Makefile target `gcp-verify-py`.

Checks (non-destructive):
- Active gcloud account
- Project and region config
- Cloud Run API access
- Cloud Storage bucket reachability (optional)
- Cloud SQL instance visibility (optional)
- Secret Manager listing

Environment:
- CLOUDSDK_CONFIG respected (set to a repo-local path to avoid $HOME perms issues)
- Optional: GCP_PROJECT_ID, GCP_REGION, GCP_BUCKET, GCP_SQL_INSTANCE
"""
from __future__ import annotations

import os
import subprocess
import sys
from dataclasses import dataclass


def run(cmd: str) -> tuple[int, str, str]:
    p = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    return p.returncode, p.stdout.strip(), p.stderr.strip()


@dataclass
class Ctx:
    project: str | None
    region: str | None
    bucket: str | None
    sql_instance: str | None
    cloudsdk_config: str | None


def ctx_from_env() -> Ctx:
    return Ctx(
        project=os.getenv("GCP_PROJECT_ID") or os.getenv("PROJECT_ID"),
        region=os.getenv("GCP_REGION") or os.getenv("REGION"),
        bucket=os.getenv("GCP_BUCKET"),
        sql_instance=os.getenv("GCP_SQL_INSTANCE") or os.getenv("SQL_INSTANCE"),
        cloudsdk_config=os.getenv("CLOUDSDK_CONFIG"),
    )


def main() -> int:
    ctx = ctx_from_env()
    print("== GCP Access Verification ==")
    if ctx.cloudsdk_config:
        print(f"CLOUDSDK_CONFIG={ctx.cloudsdk_config}")

    # 1) Active account
    rc, out, err = run("gcloud auth list --filter=status:ACTIVE --format='value(account)'")
    ok_auth = rc == 0 and bool(out)
    print(("✓" if ok_auth else "✗") + f" Active account: {out or err}")

    # 2) Project / region
    if not ctx.project:
        _rc, p_out, _ = run("gcloud config get-value core/project")
        ctx.project = p_out or None
    if not ctx.region:
        _rc, r_out, _ = run("gcloud config get-value compute/region")
        ctx.region = r_out or None
    print(("✓" if ctx.project else "✗") + f" Project: {ctx.project or '[unset]'}")
    print(("✓" if ctx.region else "✗") + f" Region:  {ctx.region or '[unset]'}")

    # 3) Core APIs / access
    apis = [
        "run.googleapis.com",
        "cloudbuild.googleapis.com",
        "artifactregistry.googleapis.com",
        "sqladmin.googleapis.com",
        "secretmanager.googleapis.com",
        "vpcaccess.googleapis.com",
        "storage.googleapis.com",
        "compute.googleapis.com",
    ]
    apis_ok = True
    for s in apis:
        rc, out, _ = run(f"gcloud services list --enabled --filter=NAME:{s} --format='value(NAME)'")
        ok = rc == 0 and s in out
        apis_ok = apis_ok and ok
        print(("✓" if ok else "✗") + f" API enabled: {s}")

    # 4) Cloud Run access
    rc, _, _ = run("gcloud run services list --limit=1 2>/dev/null")
    print(("✓" if rc == 0 else "✗") + " Cloud Run access")

    # 5) Cloud Storage (optional)
    if ctx.bucket:
        rc, out, _ = run(f"gsutil ls -b gs://{ctx.bucket} 2>/dev/null")
        print(("✓" if rc == 0 else "✗") + f" Bucket exists: gs://{ctx.bucket}")
    else:
        print("○ Bucket not provided; skip")

    # 6) Cloud SQL (optional)
    if ctx.sql_instance:
        rc, out, _ = run(
            f"gcloud sql instances describe {ctx.sql_instance} --format='value(name)' 2>/dev/null"
        )
        print(("✓" if rc == 0 and out else "✗") + f" Cloud SQL: {ctx.sql_instance}")
    else:
        print("○ SQL instance not provided; skip")

    # 7) Secret Manager
    rc, _, _ = run("gcloud secrets list --limit=1 2>/dev/null")
    print(("✓" if rc == 0 else "✗") + " Secret Manager access")

    # Summary exit code
    critical_ok = ok_auth and bool(ctx.project) and bool(ctx.region) and apis_ok
    if critical_ok:
        print("\n✅ GCP access verified. Ready to proceed.")
        return 0
    else:
        print("\n❌ GCP access verification failed. Check credentials and permissions.")
        return 2


if __name__ == "__main__":
    sys.exit(main())

