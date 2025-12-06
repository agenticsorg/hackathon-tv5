{{/*
Expand the name of the chart.
*/}}
{{- define "omega-constellation.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "omega-constellation.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "omega-constellation.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "omega-constellation.labels" -}}
helm.sh/chart: {{ include "omega-constellation.chart" . }}
{{ include "omega-constellation.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
environment: {{ .Values.global.environment }}
region: {{ .Values.global.region }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "omega-constellation.selectorLabels" -}}
app.kubernetes.io/name: {{ include "omega-constellation.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "omega-constellation.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "omega-constellation.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Constellation server labels
*/}}
{{- define "omega-constellation.constellation.labels" -}}
{{ include "omega-constellation.labels" . }}
app: constellation
component: server
{{- end }}

{{/*
Constellation server selector labels
*/}}
{{- define "omega-constellation.constellation.selectorLabels" -}}
{{ include "omega-constellation.selectorLabels" . }}
app: constellation
component: server
{{- end }}

{{/*
RuVector-Postgres labels
*/}}
{{- define "omega-constellation.postgres.labels" -}}
{{ include "omega-constellation.labels" . }}
app: ruvector-postgres
component: database
{{- end }}

{{/*
RuVector-Postgres selector labels
*/}}
{{- define "omega-constellation.postgres.selectorLabels" -}}
{{ include "omega-constellation.selectorLabels" . }}
app: ruvector-postgres
component: database
{{- end }}

{{/*
Federation worker labels
*/}}
{{- define "omega-constellation.federation.labels" -}}
{{ include "omega-constellation.labels" . }}
app: federation
component: worker
{{- end }}

{{/*
Federation worker selector labels
*/}}
{{- define "omega-constellation.federation.selectorLabels" -}}
{{ include "omega-constellation.selectorLabels" . }}
app: federation
component: worker
{{- end }}

{{/*
PostgreSQL connection URL
*/}}
{{- define "omega-constellation.postgres.url" -}}
{{- if .Values.ruvectorPostgres.auth.existingSecret }}
{{- printf "postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@%s-ruvector-postgres-primary:5432/%s?sslmode=require" (include "omega-constellation.fullname" .) .Values.ruvectorPostgres.auth.database }}
{{- else }}
{{- printf "postgres://%s:%s@%s-ruvector-postgres-primary:5432/%s?sslmode=require" .Values.ruvectorPostgres.auth.username .Values.ruvectorPostgres.auth.password (include "omega-constellation.fullname" .) .Values.ruvectorPostgres.auth.database }}
{{- end }}
{{- end }}

{{/*
PostgreSQL read replica URLs
*/}}
{{- define "omega-constellation.postgres.readUrls" -}}
{{- $fullname := include "omega-constellation.fullname" . }}
{{- $database := .Values.ruvectorPostgres.auth.database }}
{{- $urls := list }}
{{- range $i := until (int .Values.ruvectorPostgres.replicas) }}
{{- if ne $i 0 }}
{{- $url := printf "postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@%s-ruvector-postgres-%d.%s-ruvector-postgres:5432/%s?sslmode=require" $fullname $i $fullname $database }}
{{- $urls = append $urls $url }}
{{- end }}
{{- end }}
{{- join "," $urls }}
{{- end }}

{{/*
Return the appropriate apiVersion for HPA
*/}}
{{- define "omega-constellation.hpa.apiVersion" -}}
{{- if .Capabilities.APIVersions.Has "autoscaling/v2" }}
{{- print "autoscaling/v2" }}
{{- else }}
{{- print "autoscaling/v2beta2" }}
{{- end }}
{{- end }}

{{/*
Return the appropriate apiVersion for PodDisruptionBudget
*/}}
{{- define "omega-constellation.pdb.apiVersion" -}}
{{- if .Capabilities.APIVersions.Has "policy/v1" }}
{{- print "policy/v1" }}
{{- else }}
{{- print "policy/v1beta1" }}
{{- end }}
{{- end }}

{{/*
Validate configuration
*/}}
{{- define "omega-constellation.validateConfig" -}}
{{- if and .Values.ruvectorPostgres.enabled (not .Values.ruvectorPostgres.auth.password) (not .Values.ruvectorPostgres.auth.existingSecret) }}
{{- fail "ERROR: PostgreSQL password must be set via ruvectorPostgres.auth.password or ruvectorPostgres.auth.existingSecret" }}
{{- end }}
{{- if and .Values.tls.enabled (not .Values.tls.certManager.enabled) (not .Values.tls.existingSecret) }}
{{- fail "ERROR: TLS is enabled but no certificate source configured. Set tls.certManager.enabled=true or provide tls.existingSecret" }}
{{- end }}
{{- if and .Values.monitoring.grafana.enabled (not .Values.monitoring.grafana.adminPassword) (not .Values.monitoring.grafana.existingSecret) }}
{{- fail "ERROR: Grafana admin password must be set via monitoring.grafana.adminPassword or monitoring.grafana.existingSecret" }}
{{- end }}
{{- end }}

{{/*
Environment-specific annotations
*/}}
{{- define "omega-constellation.annotations" -}}
{{- if eq .Values.global.environment "production" }}
backup.velero.io/backup-volumes: postgres-data,postgres-wal
{{- end }}
{{- end }}
