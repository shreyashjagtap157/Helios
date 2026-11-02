#!/usr/bin/env ruby
# Dependency-free repository checks for the Helios design-only baseline.
require 'yaml'
require 'uri'
require 'open3'
require 'pathname'

ROOT = Pathname.new(Dir.pwd)
errors = []
checks = 0

def fail_check(errors, message)
  errors << message
end

def read_text(path, errors)
  File.read(path, encoding: 'UTF-8')
rescue StandardError => e
  errors << "Cannot read #{path}: #{e.class}: #{e.message}"
  ''
end

stdout, stderr, status = Open3.capture3('git', 'ls-files', '-z')
unless status.success?
  warn "git ls-files failed: #{stderr}"
  exit 2
end
tracked = stdout.split("\0").reject(&:empty?)
checks += 1

required = %w[
  README.md HELIOS_MASTER_SPEC.md ROADMAP.md DECISIONS.md STATUS.md
  ISSUES.md SECURITY.md CONTRIBUTING.md CHANGELOG.md NOTICE
  docs/RESEARCH_REGISTER.md docs/NON_TRAINING_COMPLIANCE.md
  .github/CODEOWNERS .github/PULL_REQUEST_TEMPLATE.md
]
required.each do |path|
  unless tracked.include?(path) && File.file?(ROOT.join(path))
    fail_check(errors, "Required project file is missing: #{path}")
  end
end
checks += required.length

forbidden_path = %r{(?:\A|/)(?:omni-lang|compiler|self-host(?:ed)?)(?:/|\z)|\.omni\z|\.ovm\z}i
tracked.each do |path|
  fail_check(errors, "Legacy implementation path remains: #{path}") if path.match?(forbidden_path)
end
checks += 1

forbidden_content = /(?:\bomni\b|omni-lang|\.omni\b|\bomnc\b|ownership semantics|systems programming language|self-hosted compiler)/i
text_extensions = %w[.md .yml .yaml .rb .py .json .toml .txt .sh .gitignore]
tracked.each do |path|
  next unless text_extensions.include?(File.extname(path).downcase) || %w[.gitignore NOTICE LICENSE CODEOWNERS].include?(File.basename(path))
  next if path == 'scripts/validate_repository.rb'
  content = read_text(ROOT.join(path), errors)
  fail_check(errors, "Legacy project reference remains in #{path}") if content.match?(forbidden_content)
end
checks += 1

# Validate issue-form YAML using Ruby's standard YAML parser and the documented form shape.
form_paths = tracked.grep(%r{\A\.github/ISSUE_TEMPLATE/[^/]+\.yml\z}).reject { |p| p.end_with?('/config.yml') }
expected_forms = %w[
  .github/ISSUE_TEMPLATE/bug_report.yml
  .github/ISSUE_TEMPLATE/feature_request.yml
  .github/ISSUE_TEMPLATE/good_first_issue_docs.yml
  .github/ISSUE_TEMPLATE/question.yml
]
(expected_forms - form_paths).each { |p| fail_check(errors, "Missing issue form: #{p}") }
(form_paths - expected_forms).each { |p| fail_check(errors, "Unreviewed issue form: #{p}") }
form_paths.each do |path|
  begin
    form = YAML.safe_load(read_text(ROOT.join(path), errors), permitted_classes: [], permitted_symbols: [], aliases: false)
    unless form.is_a?(Hash) && form['name'].is_a?(String) && form['description'].is_a?(String) && form['body'].is_a?(Array)
      fail_check(errors, "Issue form has invalid top-level structure: #{path}")
      next
    end
    ids = []
    form['body'].each_with_index do |field, index|
      unless field.is_a?(Hash) && %w[markdown textarea input dropdown checkboxes upload].include?(field['type'])
        fail_check(errors, "Unsupported or malformed issue-form field in #{path} at body[#{index}]")
        next
      end
      next if field['type'] == 'markdown'
      id = field['id']
      attrs = field['attributes']
      fail_check(errors, "Issue-form field missing id/attributes in #{path} at body[#{index}]") unless id.is_a?(String) && !id.empty? && attrs.is_a?(Hash)
      ids << id if id.is_a?(String)
      if field['validations'].is_a?(Hash) && field['validations'].key?('required') && ![true, false].include?(field['validations']['required'])
        fail_check(errors, "Issue-form required flag must be boolean in #{path} at body[#{index}]")
      end
      if %w[dropdown checkboxes].include?(field['type'])
        options = attrs['options']
        fail_check(errors, "Issue-form selection field has no options in #{path} at body[#{index}]") unless options.is_a?(Array) && !options.empty? && options.all? { |v| v.is_a?(String) && !v.empty? }
      end
    end
    duplicates = ids.group_by(&:itself).select { |_id, values| values.length > 1 }.keys
    fail_check(errors, "Duplicate issue-form field ids in #{path}: #{duplicates.join(', ')}") unless duplicates.empty?
  rescue Psych::Exception => e
    fail_check(errors, "Invalid YAML in #{path}: #{e.message.lines.first.strip}")
  end
end
checks += 1

# Validate repository-owned workflow YAML and enforce the intended read-only CI gate.
workflow_paths = tracked.grep(%r{\A\.github/workflows/[^/]+\.ya?ml\z})
if workflow_paths != ['.github/workflows/docs.yml']
  fail_check(errors, "Expected exactly one reviewed workflow (.github/workflows/docs.yml); found: #{workflow_paths.join(', ')}")
end
workflow_paths.each do |path|
  begin
    workflow = YAML.safe_load(read_text(ROOT.join(path), errors), permitted_classes: [], permitted_symbols: [], aliases: false)
    unless workflow.is_a?(Hash)
      fail_check(errors, "Workflow must be a YAML mapping: #{path}")
      next
    end
    # Psych (YAML 1.1) treats the unquoted GitHub key 'on' as boolean true.
    triggers = workflow['on'] || workflow[true]
    unless triggers.is_a?(Hash) && %w[push pull_request workflow_dispatch].all? { |event| triggers.key?(event) }
      fail_check(errors, "Workflow must define push, pull_request and workflow_dispatch triggers: #{path}")
    end
    unless triggers.dig('push', 'branches') == %w[main dev]
      fail_check(errors, "Workflow push triggers must target exactly main and dev: #{path}")
    end
    unless triggers.dig('pull_request', 'branches') == ['main']
      fail_check(errors, "Workflow pull_request trigger must target main: #{path}")
    end
    permissions = workflow['permissions']
    unless permissions.is_a?(Hash) && permissions['contents'] == 'read'
      fail_check(errors, "Workflow must declare least-privilege contents: read: #{path}")
    end
    jobs = workflow['jobs']
    unless jobs.is_a?(Hash) && jobs.keys == ['docs']
      fail_check(errors, "Workflow must contain exactly one reviewed job named docs: #{path}")
    end
    docs_job = jobs.is_a?(Hash) ? jobs['docs'] : nil
    unless docs_job.is_a?(Hash) && docs_job['name'] == 'docs'
      fail_check(errors, "Workflow must publish the stable docs job check: #{path}")
      next
    end
    unless docs_job['runs-on'] == 'ubuntu-24.04'
      fail_check(errors, "Workflow must use the explicitly pinned Ubuntu 24.04 runner: #{path}")
    end
    unless docs_job['permissions'].is_a?(Hash) && docs_job['permissions'] == { 'contents' => 'read' }
      fail_check(errors, "Workflow docs job must use only contents: read permissions: #{path}")
    end
    unless docs_job['timeout-minutes'].is_a?(Integer) && docs_job['timeout-minutes'].between?(1, 10)
      fail_check(errors, "Workflow docs job must have a 1–10 minute timeout: #{path}")
    end
    steps = docs_job['steps']
    unless steps.is_a?(Array)
      fail_check(errors, "Workflow docs job has no steps: #{path}")
      next
    end
    checkout = steps.find { |step| step.is_a?(Hash) && step['uses'].to_s.start_with?('actions/checkout@') }
    unless checkout && checkout['uses'].match?(/@(?:[0-9a-f]{40})\z/) && checkout.dig('with', 'persist-credentials') == false
      fail_check(errors, "Workflow must pin checkout to a full SHA and disable persisted credentials: #{path}")
    end
    unless steps.any? { |step| step.is_a?(Hash) && step['run'].to_s.include?('ruby scripts/validate_repository.rb') }
      fail_check(errors, "Workflow does not run the repository validator: #{path}")
    end
  rescue Psych::Exception => e
    fail_check(errors, "Invalid workflow YAML in #{path}: #{e.message.lines.first.strip}")
  end
end
checks += 1

# Validate local Markdown link targets without making CI depend on external websites.
markdown_paths = tracked.grep(/\.md\z/i)
markdown_paths.each do |source|
  content = read_text(ROOT.join(source), errors)
  content = content.gsub(/^\s*(```|~~~).*?^\s*\1\s*$/m, '')
  content.scan(/!?\[[^\]]*\]\(([^)]+)\)/).flatten.each do |raw_target|
    target = raw_target.strip.sub(/\A</, '').sub(/>\z/, '')
    next if target.empty? || target.start_with?('#')
    begin
      uri = URI.parse(target)
      next if uri.scheme || target.start_with?('//')
      local_path = URI::DEFAULT_PARSER.unescape(target.split('#', 2).first.to_s)
      next if local_path.empty?
      resolved = ROOT.join(File.dirname(source), local_path).cleanpath
      unless resolved.file? || resolved.directory?
        fail_check(errors, "Broken local Markdown link in #{source}: #{target}")
      end
    rescue URI::InvalidURIError
      fail_check(errors, "Invalid Markdown link target in #{source}: #{target}")
    end
  end
end
checks += 1

# Verify the master requirement register has unique identifiers and all non-training gates.
spec_path = ROOT.join('HELIOS_MASTER_SPEC.md')
if spec_path.file?
  spec = read_text(spec_path, errors)
  ids = spec.scan(/^- (HEL-[A-Z]+-\d{3}):/).flatten
  duplicates = ids.group_by(&:itself).select { |_id, values| values.length > 1 }.keys
  fail_check(errors, "Duplicate requirement identifiers: #{duplicates.join(', ')}") unless duplicates.empty?
  (1..10).each do |n|
    id = format('HEL-NTR-%03d', n)
    fail_check(errors, "Missing non-training requirement: #{id}") unless ids.include?(id)
  end
end
checks += 1

# The repository remains documentation-only until a separately approved implementation start.
status_doc = ROOT.join('STATUS.md')
if status_doc.file?
  state = read_text(status_doc, errors)
  fail_check(errors, 'STATUS.md must explicitly state that implementation has not started') unless state.match?(/\*\*Implementation:\*\* Not started/i)
end
checks += 1

if errors.empty?
  puts "Helios repository validation passed (#{checks} validation groups; #{tracked.length} tracked files)."
  exit 0
end
warn "Helios repository validation failed with #{errors.length} error(s):"
errors.each { |error| warn "- #{error}" }
exit 1
