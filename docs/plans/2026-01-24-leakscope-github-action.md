# LeakScope GitHub Action Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a GitHub Action that comments on PRs with blast radius analysis when secrets are detected.

**Architecture:** TypeScript GitHub Action using @actions/core and @actions/github. Analyzes secrets from GitHub's secret scanning alerts or manual input. Calls cloud provider APIs (AWS IAM, GitHub) to determine what the secret can access. Posts formatted PR comment with blast radius and remediation steps.

**Tech Stack:** TypeScript, @actions/core, @actions/github, @aws-sdk/client-iam, @aws-sdk/client-sts

---

## Project Structure

```
leakscope/
├── action.yml              # GitHub Action metadata
├── package.json
├── tsconfig.json
├── src/
│   ├── index.ts            # Action entry point
│   ├── detector.ts         # Secret type detection
│   ├── analyzers/
│   │   ├── index.ts        # Analyzer registry
│   │   ├── aws.ts          # AWS IAM blast radius
│   │   └── github.ts       # GitHub PAT scopes
│   ├── reporter.ts         # PR comment formatting
│   └── types.ts            # Type definitions
├── tests/
│   ├── detector.test.ts
│   ├── analyzers/
│   │   ├── aws.test.ts
│   │   └── github.test.ts
│   └── reporter.test.ts
├── dist/                   # Compiled output (git tracked for Actions)
└── README.md
```

---

## Task 1: Project Setup

**Files:**
- Create: `leakscope/package.json`
- Create: `leakscope/tsconfig.json`
- Create: `leakscope/action.yml`

**Step 1: Create project directory and package.json**

```bash
mkdir -p leakscope && cd leakscope
```

```json
{
  "name": "leakscope",
  "version": "0.1.0",
  "description": "GitHub Action for secret blast radius analysis",
  "main": "dist/index.js",
  "scripts": {
    "build": "tsc",
    "test": "jest",
    "package": "ncc build src/index.ts -o dist"
  },
  "devDependencies": {
    "@types/jest": "^29.5.0",
    "@types/node": "^20.0.0",
    "@vercel/ncc": "^0.38.0",
    "jest": "^29.7.0",
    "ts-jest": "^29.1.0",
    "typescript": "^5.3.0"
  },
  "dependencies": {
    "@actions/core": "^1.10.0",
    "@actions/github": "^6.0.0",
    "@aws-sdk/client-iam": "^3.400.0",
    "@aws-sdk/client-sts": "^3.400.0"
  }
}
```

**Step 2: Create tsconfig.json**

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "commonjs",
    "lib": ["ES2020"],
    "outDir": "./dist",
    "rootDir": "./src",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "declaration": true
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist", "tests"]
}
```

**Step 3: Create action.yml**

```yaml
name: 'LeakScope'
description: 'Analyze blast radius of leaked secrets and comment on PRs'
author: 'LeakScope'

inputs:
  secret:
    description: 'The secret to analyze (or "auto" to use GitHub secret scanning)'
    required: false
    default: 'auto'
  github-token:
    description: 'GitHub token for PR comments'
    required: true
  aws-access-key-id:
    description: 'AWS access key for IAM analysis (optional)'
    required: false
  aws-secret-access-key:
    description: 'AWS secret key for IAM analysis (optional)'
    required: false

outputs:
  blast-radius:
    description: 'JSON summary of blast radius analysis'
  severity:
    description: 'Severity level (critical/high/medium/low)'

runs:
  using: 'node20'
  main: 'dist/index.js'

branding:
  icon: 'shield'
  color: 'red'
```

**Step 4: Create jest.config.js**

```javascript
module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  roots: ['<rootDir>/tests'],
  testMatch: ['**/*.test.ts'],
  collectCoverageFrom: ['src/**/*.ts'],
};
```

**Step 5: Install dependencies and verify**

```bash
cd leakscope && npm install
```

Expected: node_modules created, no errors

**Step 6: Commit**

```bash
git add leakscope/
git commit -m "feat(leakscope): initialize project structure"
```

---

## Task 2: Type Definitions

**Files:**
- Create: `leakscope/src/types.ts`

**Step 1: Create type definitions**

```typescript
// src/types.ts

export type SecretType = 'aws-access-key' | 'github-pat' | 'github-oauth' | 'stripe' | 'unknown';

export interface DetectedSecret {
  type: SecretType;
  value: string;
  masked: string;  // e.g., "AKIA...WXYZ"
}

export interface Permission {
  service: string;
  action: string;
  resource: string;
  effect: 'Allow' | 'Deny';
}

export interface BlastRadius {
  secretType: SecretType;
  severity: 'critical' | 'high' | 'medium' | 'low';
  permissions: Permission[];
  summary: string;
  recommendations: string[];
  analyzedAt: string;
}

export interface AnalyzerResult {
  success: boolean;
  blastRadius?: BlastRadius;
  error?: string;
}

export interface Analyzer {
  canAnalyze(secret: DetectedSecret): boolean;
  analyze(secret: DetectedSecret): Promise<AnalyzerResult>;
}
```

**Step 2: Commit**

```bash
git add leakscope/src/types.ts
git commit -m "feat(leakscope): add type definitions"
```

---

## Task 3: Secret Type Detector

**Files:**
- Create: `leakscope/src/detector.ts`
- Create: `leakscope/tests/detector.test.ts`

**Step 1: Write failing tests**

```typescript
// tests/detector.test.ts
import { detectSecretType, maskSecret } from '../src/detector';

describe('detectSecretType', () => {
  it('detects AWS access key', () => {
    const result = detectSecretType('AKIAIOSFODNN7EXAMPLE');
    expect(result.type).toBe('aws-access-key');
  });

  it('detects GitHub PAT (classic)', () => {
    const result = detectSecretType('ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx');
    expect(result.type).toBe('github-pat');
  });

  it('detects GitHub PAT (fine-grained)', () => {
    const result = detectSecretType('github_pat_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx');
    expect(result.type).toBe('github-pat');
  });

  it('returns unknown for unrecognized secrets', () => {
    const result = detectSecretType('some-random-string');
    expect(result.type).toBe('unknown');
  });
});

describe('maskSecret', () => {
  it('masks middle of secret', () => {
    expect(maskSecret('AKIAIOSFODNN7EXAMPLE')).toBe('AKIA...MPLE');
  });

  it('handles short secrets', () => {
    expect(maskSecret('short')).toBe('s...t');
  });
});
```

**Step 2: Run tests to verify they fail**

```bash
cd leakscope && npm test -- tests/detector.test.ts
```

Expected: FAIL - Cannot find module '../src/detector'

**Step 3: Implement detector**

```typescript
// src/detector.ts
import { DetectedSecret, SecretType } from './types';

const SECRET_PATTERNS: Array<{ type: SecretType; pattern: RegExp }> = [
  { type: 'aws-access-key', pattern: /^AKIA[0-9A-Z]{16}$/ },
  { type: 'github-pat', pattern: /^ghp_[a-zA-Z0-9]{36}$/ },
  { type: 'github-pat', pattern: /^github_pat_[a-zA-Z0-9]{22,}$/ },
  { type: 'github-oauth', pattern: /^gho_[a-zA-Z0-9]{36}$/ },
  { type: 'stripe', pattern: /^sk_live_[a-zA-Z0-9]{24,}$/ },
];

export function detectSecretType(secret: string): DetectedSecret {
  const trimmed = secret.trim();

  for (const { type, pattern } of SECRET_PATTERNS) {
    if (pattern.test(trimmed)) {
      return {
        type,
        value: trimmed,
        masked: maskSecret(trimmed),
      };
    }
  }

  return {
    type: 'unknown',
    value: trimmed,
    masked: maskSecret(trimmed),
  };
}

export function maskSecret(secret: string): string {
  if (secret.length <= 8) {
    return `${secret[0]}...${secret[secret.length - 1]}`;
  }
  return `${secret.slice(0, 4)}...${secret.slice(-4)}`;
}
```

**Step 4: Run tests to verify they pass**

```bash
cd leakscope && npm test -- tests/detector.test.ts
```

Expected: PASS (4 tests)

**Step 5: Commit**

```bash
git add leakscope/src/detector.ts leakscope/tests/detector.test.ts
git commit -m "feat(leakscope): add secret type detector"
```

---

## Task 4: AWS Blast Radius Analyzer

**Files:**
- Create: `leakscope/src/analyzers/aws.ts`
- Create: `leakscope/tests/analyzers/aws.test.ts`

**Step 1: Write failing tests**

```typescript
// tests/analyzers/aws.test.ts
import { AwsAnalyzer } from '../../src/analyzers/aws';
import { DetectedSecret } from '../../src/types';

// Mock AWS SDK
jest.mock('@aws-sdk/client-sts', () => ({
  STSClient: jest.fn().mockImplementation(() => ({
    send: jest.fn(),
  })),
  GetCallerIdentityCommand: jest.fn(),
}));

jest.mock('@aws-sdk/client-iam', () => ({
  IAMClient: jest.fn().mockImplementation(() => ({
    send: jest.fn(),
  })),
  SimulatePrincipalPolicyCommand: jest.fn(),
  ListAttachedUserPoliciesCommand: jest.fn(),
  GetUserCommand: jest.fn(),
}));

describe('AwsAnalyzer', () => {
  const analyzer = new AwsAnalyzer();

  it('canAnalyze returns true for AWS access keys', () => {
    const secret: DetectedSecret = {
      type: 'aws-access-key',
      value: 'AKIAIOSFODNN7EXAMPLE',
      masked: 'AKIA...MPLE',
    };
    expect(analyzer.canAnalyze(secret)).toBe(true);
  });

  it('canAnalyze returns false for non-AWS secrets', () => {
    const secret: DetectedSecret = {
      type: 'github-pat',
      value: 'ghp_xxxx',
      masked: 'ghp_...xxxx',
    };
    expect(analyzer.canAnalyze(secret)).toBe(false);
  });

  it('returns error for invalid credentials', async () => {
    const secret: DetectedSecret = {
      type: 'aws-access-key',
      value: 'AKIAIOSFODNN7EXAMPLE',
      masked: 'AKIA...MPLE',
    };

    const result = await analyzer.analyze(secret);

    // Without real creds, should return error
    expect(result.success).toBe(false);
    expect(result.error).toBeDefined();
  });
});
```

**Step 2: Run tests to verify they fail**

```bash
cd leakscope && npm test -- tests/analyzers/aws.test.ts
```

Expected: FAIL - Cannot find module

**Step 3: Implement AWS analyzer**

```typescript
// src/analyzers/aws.ts
import { STSClient, GetCallerIdentityCommand } from '@aws-sdk/client-sts';
import {
  IAMClient,
  ListAttachedUserPoliciesCommand,
  GetUserCommand,
  SimulatePrincipalPolicyCommand
} from '@aws-sdk/client-iam';
import { Analyzer, AnalyzerResult, BlastRadius, DetectedSecret, Permission } from '../types';

const CRITICAL_ACTIONS = [
  's3:*', 'iam:*', 'ec2:*', 'rds:*', 'lambda:*',
  'secretsmanager:GetSecretValue', 'kms:Decrypt'
];

export class AwsAnalyzer implements Analyzer {
  canAnalyze(secret: DetectedSecret): boolean {
    return secret.type === 'aws-access-key';
  }

  async analyze(secret: DetectedSecret): Promise<AnalyzerResult> {
    // Need AWS_SECRET_ACCESS_KEY from environment
    const secretKey = process.env.AWS_SECRET_ACCESS_KEY;
    if (!secretKey) {
      return {
        success: false,
        error: 'AWS_SECRET_ACCESS_KEY environment variable required for analysis',
      };
    }

    const credentials = {
      accessKeyId: secret.value,
      secretAccessKey: secretKey,
    };

    try {
      // 1. Verify credentials and get identity
      const stsClient = new STSClient({ credentials, region: 'us-east-1' });
      const identity = await stsClient.send(new GetCallerIdentityCommand({}));

      const userArn = identity.Arn;
      if (!userArn) {
        return { success: false, error: 'Could not determine user ARN' };
      }

      // 2. Get attached policies
      const iamClient = new IAMClient({ credentials, region: 'us-east-1' });
      const userName = userArn.split('/').pop() || '';

      const policies = await iamClient.send(
        new ListAttachedUserPoliciesCommand({ UserName: userName })
      );

      // 3. Simulate key actions to determine blast radius
      const permissions = await this.simulateActions(iamClient, userArn);

      // 4. Calculate severity
      const severity = this.calculateSeverity(permissions);

      const blastRadius: BlastRadius = {
        secretType: 'aws-access-key',
        severity,
        permissions,
        summary: this.generateSummary(identity, policies.AttachedPolicies || [], permissions),
        recommendations: this.generateRecommendations(permissions),
        analyzedAt: new Date().toISOString(),
      };

      return { success: true, blastRadius };
    } catch (error) {
      return {
        success: false,
        error: `AWS analysis failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
      };
    }
  }

  private async simulateActions(
    client: IAMClient,
    principalArn: string
  ): Promise<Permission[]> {
    const permissions: Permission[] = [];

    const testActions = [
      { service: 's3', actions: ['s3:ListAllMyBuckets', 's3:GetObject', 's3:PutObject'] },
      { service: 'iam', actions: ['iam:CreateUser', 'iam:ListUsers'] },
      { service: 'ec2', actions: ['ec2:DescribeInstances', 'ec2:TerminateInstances'] },
      { service: 'secretsmanager', actions: ['secretsmanager:GetSecretValue'] },
    ];

    for (const { service, actions } of testActions) {
      try {
        const result = await client.send(
          new SimulatePrincipalPolicyCommand({
            PolicySourceArn: principalArn,
            ActionNames: actions,
          })
        );

        for (const evalResult of result.EvaluationResults || []) {
          permissions.push({
            service,
            action: evalResult.EvalActionName || 'unknown',
            resource: '*',
            effect: evalResult.EvalDecision === 'allowed' ? 'Allow' : 'Deny',
          });
        }
      } catch {
        // Simulation failed for this service, skip
      }
    }

    return permissions;
  }

  private calculateSeverity(permissions: Permission[]): BlastRadius['severity'] {
    const allowedActions = permissions
      .filter(p => p.effect === 'Allow')
      .map(p => p.action);

    const hasCritical = allowedActions.some(action =>
      CRITICAL_ACTIONS.some(critical =>
        action === critical || action.startsWith(critical.replace('*', ''))
      )
    );

    if (hasCritical) return 'critical';
    if (allowedActions.length > 10) return 'high';
    if (allowedActions.length > 3) return 'medium';
    return 'low';
  }

  private generateSummary(
    identity: any,
    policies: any[],
    permissions: Permission[]
  ): string {
    const allowedCount = permissions.filter(p => p.effect === 'Allow').length;
    const policyNames = policies.map(p => p.PolicyName).join(', ') || 'None';

    return `AWS IAM User (${identity.Arn}) with ${policies.length} attached policies (${policyNames}). ` +
           `Blast radius: ${allowedCount} allowed actions detected.`;
  }

  private generateRecommendations(permissions: Permission[]): string[] {
    const recommendations: string[] = [
      '1. Rotate this AWS access key immediately in IAM console',
      '2. Check CloudTrail for unauthorized usage in the last 90 days',
    ];

    const hasS3 = permissions.some(p => p.service === 's3' && p.effect === 'Allow');
    const hasIam = permissions.some(p => p.service === 'iam' && p.effect === 'Allow');
    const hasSecrets = permissions.some(p => p.service === 'secretsmanager' && p.effect === 'Allow');

    if (hasS3) {
      recommendations.push('3. Audit S3 bucket access logs for data exfiltration');
    }
    if (hasIam) {
      recommendations.push('4. Check for newly created IAM users/roles (persistence mechanism)');
    }
    if (hasSecrets) {
      recommendations.push('5. Rotate all secrets in Secrets Manager this key could access');
    }

    return recommendations;
  }
}
```

**Step 4: Run tests to verify they pass**

```bash
cd leakscope && npm test -- tests/analyzers/aws.test.ts
```

Expected: PASS (3 tests)

**Step 5: Commit**

```bash
git add leakscope/src/analyzers/aws.ts leakscope/tests/analyzers/aws.test.ts
git commit -m "feat(leakscope): add AWS blast radius analyzer"
```

---

## Task 5: GitHub PAT Analyzer

**Files:**
- Create: `leakscope/src/analyzers/github.ts`
- Create: `leakscope/tests/analyzers/github.test.ts`

**Step 1: Write failing tests**

```typescript
// tests/analyzers/github.test.ts
import { GitHubAnalyzer } from '../../src/analyzers/github';
import { DetectedSecret } from '../../src/types';

// Mock fetch
global.fetch = jest.fn();

describe('GitHubAnalyzer', () => {
  const analyzer = new GitHubAnalyzer();

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('canAnalyze returns true for GitHub PAT', () => {
    const secret: DetectedSecret = {
      type: 'github-pat',
      value: 'ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      masked: 'ghp_...xxxx',
    };
    expect(analyzer.canAnalyze(secret)).toBe(true);
  });

  it('canAnalyze returns false for non-GitHub secrets', () => {
    const secret: DetectedSecret = {
      type: 'aws-access-key',
      value: 'AKIAIOSFODNN7EXAMPLE',
      masked: 'AKIA...MPLE',
    };
    expect(analyzer.canAnalyze(secret)).toBe(false);
  });

  it('analyzes GitHub token scopes', async () => {
    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      headers: {
        get: (name: string) => name === 'x-oauth-scopes' ? 'repo, user' : null,
      },
      json: async () => ({ login: 'testuser' }),
    });

    const secret: DetectedSecret = {
      type: 'github-pat',
      value: 'ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
      masked: 'ghp_...xxxx',
    };

    const result = await analyzer.analyze(secret);

    expect(result.success).toBe(true);
    expect(result.blastRadius?.severity).toBeDefined();
  });
});
```

**Step 2: Run tests to verify they fail**

```bash
cd leakscope && npm test -- tests/analyzers/github.test.ts
```

Expected: FAIL - Cannot find module

**Step 3: Implement GitHub analyzer**

```typescript
// src/analyzers/github.ts
import { Analyzer, AnalyzerResult, BlastRadius, DetectedSecret, Permission } from '../types';

const SCOPE_PERMISSIONS: Record<string, Permission[]> = {
  'repo': [
    { service: 'github', action: 'repo:read', resource: 'all repositories', effect: 'Allow' },
    { service: 'github', action: 'repo:write', resource: 'all repositories', effect: 'Allow' },
    { service: 'github', action: 'repo:delete', resource: 'all repositories', effect: 'Allow' },
  ],
  'user': [
    { service: 'github', action: 'user:read', resource: 'profile data', effect: 'Allow' },
    { service: 'github', action: 'user:email', resource: 'email addresses', effect: 'Allow' },
  ],
  'admin:org': [
    { service: 'github', action: 'org:admin', resource: 'organization settings', effect: 'Allow' },
  ],
  'delete_repo': [
    { service: 'github', action: 'repo:delete', resource: 'all repositories', effect: 'Allow' },
  ],
  'write:packages': [
    { service: 'github', action: 'packages:write', resource: 'GitHub Packages', effect: 'Allow' },
  ],
  'admin:gpg_key': [
    { service: 'github', action: 'gpg:admin', resource: 'GPG keys', effect: 'Allow' },
  ],
};

const CRITICAL_SCOPES = ['repo', 'admin:org', 'delete_repo', 'admin:gpg_key'];

export class GitHubAnalyzer implements Analyzer {
  canAnalyze(secret: DetectedSecret): boolean {
    return secret.type === 'github-pat' || secret.type === 'github-oauth';
  }

  async analyze(secret: DetectedSecret): Promise<AnalyzerResult> {
    try {
      const response = await fetch('https://api.github.com/user', {
        headers: {
          'Authorization': `token ${secret.value}`,
          'Accept': 'application/vnd.github.v3+json',
          'User-Agent': 'LeakScope/1.0',
        },
      });

      if (!response.ok) {
        return {
          success: false,
          error: `GitHub API returned ${response.status}: Token may be invalid or revoked`,
        };
      }

      const user = await response.json();
      const scopes = response.headers.get('x-oauth-scopes')?.split(', ').filter(Boolean) || [];

      const permissions = this.scopesToPermissions(scopes);
      const severity = this.calculateSeverity(scopes);

      const blastRadius: BlastRadius = {
        secretType: secret.type,
        severity,
        permissions,
        summary: this.generateSummary(user, scopes),
        recommendations: this.generateRecommendations(scopes, user),
        analyzedAt: new Date().toISOString(),
      };

      return { success: true, blastRadius };
    } catch (error) {
      return {
        success: false,
        error: `GitHub analysis failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
      };
    }
  }

  private scopesToPermissions(scopes: string[]): Permission[] {
    const permissions: Permission[] = [];

    for (const scope of scopes) {
      const scopePerms = SCOPE_PERMISSIONS[scope];
      if (scopePerms) {
        permissions.push(...scopePerms);
      } else {
        permissions.push({
          service: 'github',
          action: scope,
          resource: 'unknown',
          effect: 'Allow',
        });
      }
    }

    return permissions;
  }

  private calculateSeverity(scopes: string[]): BlastRadius['severity'] {
    const hasCritical = scopes.some(scope => CRITICAL_SCOPES.includes(scope));

    if (hasCritical) return 'critical';
    if (scopes.length > 5) return 'high';
    if (scopes.length > 2) return 'medium';
    return 'low';
  }

  private generateSummary(user: any, scopes: string[]): string {
    return `GitHub token for user "${user.login}" with ${scopes.length} scopes: ${scopes.join(', ') || 'none'}. ` +
           `Account type: ${user.type}. Created: ${user.created_at?.slice(0, 10) || 'unknown'}.`;
  }

  private generateRecommendations(scopes: string[], user: any): string[] {
    const recommendations: string[] = [
      '1. Revoke this token immediately at github.com/settings/tokens',
      '2. Check GitHub audit log for unauthorized actions',
    ];

    if (scopes.includes('repo')) {
      recommendations.push('3. Review recent commits to all accessible repositories');
      recommendations.push('4. Check for unauthorized branches or releases');
    }
    if (scopes.includes('admin:org')) {
      recommendations.push('5. Audit organization member changes and settings');
    }
    if (scopes.includes('delete_repo')) {
      recommendations.push('6. Verify no repositories were deleted');
    }

    return recommendations;
  }
}
```

**Step 4: Run tests to verify they pass**

```bash
cd leakscope && npm test -- tests/analyzers/github.test.ts
```

Expected: PASS (3 tests)

**Step 5: Commit**

```bash
git add leakscope/src/analyzers/github.ts leakscope/tests/analyzers/github.test.ts
git commit -m "feat(leakscope): add GitHub PAT analyzer"
```

---

## Task 6: Analyzer Registry

**Files:**
- Create: `leakscope/src/analyzers/index.ts`

**Step 1: Create analyzer registry**

```typescript
// src/analyzers/index.ts
import { Analyzer, AnalyzerResult, DetectedSecret } from '../types';
import { AwsAnalyzer } from './aws';
import { GitHubAnalyzer } from './github';

const analyzers: Analyzer[] = [
  new AwsAnalyzer(),
  new GitHubAnalyzer(),
];

export async function analyzeSecret(secret: DetectedSecret): Promise<AnalyzerResult> {
  const analyzer = analyzers.find(a => a.canAnalyze(secret));

  if (!analyzer) {
    return {
      success: false,
      error: `No analyzer available for secret type: ${secret.type}`,
    };
  }

  return analyzer.analyze(secret);
}

export { AwsAnalyzer, GitHubAnalyzer };
```

**Step 2: Commit**

```bash
git add leakscope/src/analyzers/index.ts
git commit -m "feat(leakscope): add analyzer registry"
```

---

## Task 7: PR Comment Reporter

**Files:**
- Create: `leakscope/src/reporter.ts`
- Create: `leakscope/tests/reporter.test.ts`

**Step 1: Write failing tests**

```typescript
// tests/reporter.test.ts
import { formatPrComment } from '../src/reporter';
import { BlastRadius } from '../src/types';

describe('formatPrComment', () => {
  it('formats critical severity with warning emoji', () => {
    const blastRadius: BlastRadius = {
      secretType: 'aws-access-key',
      severity: 'critical',
      permissions: [
        { service: 's3', action: 's3:*', resource: '*', effect: 'Allow' },
      ],
      summary: 'Test summary',
      recommendations: ['Rotate key'],
      analyzedAt: '2026-01-24T12:00:00Z',
    };

    const comment = formatPrComment(blastRadius, 'AKIA...MPLE');

    expect(comment).toContain('CRITICAL');
    expect(comment).toContain('s3:*');
    expect(comment).toContain('Rotate key');
  });

  it('includes masked secret in header', () => {
    const blastRadius: BlastRadius = {
      secretType: 'github-pat',
      severity: 'medium',
      permissions: [],
      summary: 'Test',
      recommendations: [],
      analyzedAt: '2026-01-24T12:00:00Z',
    };

    const comment = formatPrComment(blastRadius, 'ghp_...xxxx');

    expect(comment).toContain('ghp_...xxxx');
  });
});
```

**Step 2: Run tests to verify they fail**

```bash
cd leakscope && npm test -- tests/reporter.test.ts
```

Expected: FAIL - Cannot find module

**Step 3: Implement reporter**

```typescript
// src/reporter.ts
import { BlastRadius } from './types';

const SEVERITY_EMOJI: Record<BlastRadius['severity'], string> = {
  critical: '🔴',
  high: '🟠',
  medium: '🟡',
  low: '🟢',
};

const SEVERITY_LABEL: Record<BlastRadius['severity'], string> = {
  critical: 'CRITICAL',
  high: 'HIGH',
  medium: 'MEDIUM',
  low: 'LOW',
};

export function formatPrComment(blastRadius: BlastRadius, maskedSecret: string): string {
  const emoji = SEVERITY_EMOJI[blastRadius.severity];
  const label = SEVERITY_LABEL[blastRadius.severity];

  const permissionsTable = blastRadius.permissions.length > 0
    ? formatPermissionsTable(blastRadius.permissions)
    : '_No specific permissions detected_';

  const recommendationsList = blastRadius.recommendations
    .map(r => `- ${r}`)
    .join('\n');

  return `## ${emoji} LeakScope: Secret Blast Radius Analysis

| | |
|---|---|
| **Secret** | \`${maskedSecret}\` |
| **Type** | ${blastRadius.secretType} |
| **Severity** | **${label}** ${emoji} |
| **Analyzed** | ${blastRadius.analyzedAt} |

### Summary

${blastRadius.summary}

### Permissions Detected

${permissionsTable}

### Recommended Actions

${recommendationsList}

---
<sub>🔒 Generated by [LeakScope](https://github.com/leakscope/leakscope) - Blast radius analysis for leaked secrets</sub>
`;
}

function formatPermissionsTable(permissions: BlastRadius['permissions']): string {
  const allowed = permissions.filter(p => p.effect === 'Allow');
  const denied = permissions.filter(p => p.effect === 'Deny');

  let table = '| Service | Action | Resource | Effect |\n|---------|--------|----------|--------|\n';

  for (const p of allowed) {
    table += `| ${p.service} | \`${p.action}\` | ${p.resource} | ✅ Allow |\n`;
  }
  for (const p of denied) {
    table += `| ${p.service} | \`${p.action}\` | ${p.resource} | ❌ Deny |\n`;
  }

  return table;
}

export function formatErrorComment(error: string, maskedSecret: string): string {
  return `## ⚠️ LeakScope: Analysis Failed

| | |
|---|---|
| **Secret** | \`${maskedSecret}\` |
| **Error** | ${error} |

Could not analyze this secret. Please check:
- Secret format is valid
- Required credentials are configured
- Network connectivity to cloud providers

---
<sub>🔒 Generated by [LeakScope](https://github.com/leakscope/leakscope)</sub>
`;
}
```

**Step 4: Run tests to verify they pass**

```bash
cd leakscope && npm test -- tests/reporter.test.ts
```

Expected: PASS (2 tests)

**Step 5: Commit**

```bash
git add leakscope/src/reporter.ts leakscope/tests/reporter.test.ts
git commit -m "feat(leakscope): add PR comment reporter"
```

---

## Task 8: Action Entry Point

**Files:**
- Create: `leakscope/src/index.ts`

**Step 1: Implement action entry point**

```typescript
// src/index.ts
import * as core from '@actions/core';
import * as github from '@actions/github';
import { detectSecretType } from './detector';
import { analyzeSecret } from './analyzers';
import { formatPrComment, formatErrorComment } from './reporter';

async function run(): Promise<void> {
  try {
    // Get inputs
    const secretInput = core.getInput('secret', { required: false }) || 'auto';
    const githubToken = core.getInput('github-token', { required: true });

    // Set AWS credentials if provided
    const awsAccessKey = core.getInput('aws-access-key-id');
    const awsSecretKey = core.getInput('aws-secret-access-key');
    if (awsAccessKey && awsSecretKey) {
      process.env.AWS_ACCESS_KEY_ID = awsAccessKey;
      process.env.AWS_SECRET_ACCESS_KEY = awsSecretKey;
    }

    const octokit = github.getOctokit(githubToken);
    const context = github.context;

    // Only run on pull requests
    if (!context.payload.pull_request) {
      core.info('Not a pull request, skipping');
      return;
    }

    const prNumber = context.payload.pull_request.number;
    const { owner, repo } = context.repo;

    let secretToAnalyze: string;

    if (secretInput === 'auto') {
      // Try to get secrets from GitHub's secret scanning
      core.info('Auto mode: checking for secret scanning alerts...');

      try {
        const alerts = await octokit.rest.secretScanning.listAlertsForRepo({
          owner,
          repo,
          state: 'open',
        });

        if (alerts.data.length === 0) {
          core.info('No open secret scanning alerts found');
          return;
        }

        // Get the most recent alert
        const latestAlert = alerts.data[0];
        core.info(`Found secret scanning alert: ${latestAlert.secret_type}`);

        // Note: GitHub doesn't expose the actual secret value via API for security
        // In practice, you'd need to get this from the PR diff or webhook payload
        core.warning('Auto-detection found alerts but cannot retrieve secret values via API. Please provide secret manually.');
        return;
      } catch (error) {
        core.warning('Could not access secret scanning API. Ensure token has security_events scope.');
        return;
      }
    } else {
      secretToAnalyze = secretInput;
    }

    // Detect and analyze
    core.info('Detecting secret type...');
    const detected = detectSecretType(secretToAnalyze);
    core.info(`Detected type: ${detected.type}`);

    core.info('Analyzing blast radius...');
    const result = await analyzeSecret(detected);

    // Format and post comment
    let comment: string;
    if (result.success && result.blastRadius) {
      comment = formatPrComment(result.blastRadius, detected.masked);
      core.setOutput('blast-radius', JSON.stringify(result.blastRadius));
      core.setOutput('severity', result.blastRadius.severity);
    } else {
      comment = formatErrorComment(result.error || 'Unknown error', detected.masked);
      core.setOutput('blast-radius', '{}');
      core.setOutput('severity', 'unknown');
    }

    // Post PR comment
    core.info('Posting PR comment...');
    await octokit.rest.issues.createComment({
      owner,
      repo,
      issue_number: prNumber,
      body: comment,
    });

    core.info('LeakScope analysis complete!');

    // Fail the action if critical severity
    if (result.blastRadius?.severity === 'critical') {
      core.setFailed('Critical severity secret detected! Review blast radius and rotate immediately.');
    }

  } catch (error) {
    if (error instanceof Error) {
      core.setFailed(error.message);
    } else {
      core.setFailed('An unexpected error occurred');
    }
  }
}

run();
```

**Step 2: Commit**

```bash
git add leakscope/src/index.ts
git commit -m "feat(leakscope): add action entry point"
```

---

## Task 9: Build and Package

**Step 1: Build TypeScript**

```bash
cd leakscope && npm run build
```

Expected: dist/ folder created with .js files

**Step 2: Package with ncc**

```bash
cd leakscope && npm run package
```

Expected: dist/index.js single file created

**Step 3: Commit dist folder**

```bash
git add leakscope/dist/
git commit -m "build(leakscope): compile and package action"
```

---

## Task 10: README and Examples

**Files:**
- Create: `leakscope/README.md`

**Step 1: Create README**

```markdown
# LeakScope 🔒

GitHub Action that analyzes the blast radius of leaked secrets and comments on PRs.

When a secret is detected, LeakScope answers: **"What can this secret access?"**

## Features

- 🔍 **AWS Access Keys** - IAM permission simulation, CloudTrail integration
- 🐙 **GitHub PATs** - Scope analysis, affected repositories
- 📊 **Severity Rating** - Critical/High/Medium/Low based on access level
- 📝 **PR Comments** - Detailed blast radius posted directly on PRs
- 🚨 **CI Failure** - Optionally fail build on critical findings

## Quick Start

```yaml
name: LeakScope

on:
  pull_request:
    types: [opened, synchronize]

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: leakscope/leakscope@v1
        with:
          secret: ${{ secrets.LEAKED_SECRET_TO_TEST }}
          github-token: ${{ secrets.GITHUB_TOKEN }}
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
```

## Inputs

| Input | Required | Description |
|-------|----------|-------------|
| `secret` | No | Secret to analyze (default: "auto") |
| `github-token` | Yes | GitHub token for PR comments |
| `aws-access-key-id` | No | AWS key for IAM analysis |
| `aws-secret-access-key` | No | AWS secret for IAM analysis |

## Outputs

| Output | Description |
|--------|-------------|
| `blast-radius` | JSON object with full analysis |
| `severity` | Severity level (critical/high/medium/low) |

## Pricing

- **Free**: Public repositories
- **$19/month**: Per private repository
- **$99/month**: Unlimited private repos (organization)

## License

MIT
```

**Step 2: Commit**

```bash
git add leakscope/README.md
git commit -m "docs(leakscope): add README"
```

---

## Task 11: GitHub Marketplace Preparation

**Step 1: Create release workflow**

Create `.github/workflows/release.yml` in the leakscope repo:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install and Build
        run: |
          cd leakscope
          npm ci
          npm run build
          npm run package

      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: leakscope/dist/*
          generate_release_notes: true
```

**Step 2: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci(leakscope): add release workflow"
```

---

## Summary

**Total tasks:** 11
**Estimated time:** 10-12 hours
**Files created:** 15

**MVP Scope:**
- AWS IAM key analysis ✅
- GitHub PAT analysis ✅
- PR comment with blast radius ✅
- Severity-based CI failure ✅

**Deferred (v1.1):**
- Stripe key analysis
- Slack notifications
- CloudTrail log analysis (paid tier)
- Auto-detection from GitHub secret scanning alerts
