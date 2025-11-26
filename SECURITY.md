# Security Policy

## Supported Versions

We actively support the following versions with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 0.38.x  | :white_check_mark: |
| < 0.38  | :x:                |

We recommend always running the latest version to ensure you have the most recent security patches.

## Reporting a Vulnerability

We take security vulnerabilities seriously. If you discover a security issue, please report it responsibly.

### How to Report

**Do NOT open a public GitHub issue for security vulnerabilities.**

Instead, please report security vulnerabilities via:

- **Email**: mackman42@outlook.com
- **GitHub**: Contact [@mack42](https://github.com/mack42) directly

### What to Include

When reporting a vulnerability, please include:

1. **Description**: A clear description of the vulnerability
2. **Impact**: The potential impact and severity
3. **Steps to Reproduce**: Detailed steps to reproduce the issue
4. **Affected Versions**: Which versions are affected
5. **Proof of Concept**: Code or screenshots demonstrating the vulnerability (if possible)
6. **Suggested Fix**: Any recommendations for fixing the issue (optional)

### Response Timeline

- **Initial Response**: Within 48 hours of receiving your report
- **Status Update**: Within 7 days with our assessment
- **Resolution Target**: Critical vulnerabilities within 30 days

### What to Expect

1. **Acknowledgment**: We will confirm receipt of your report
2. **Investigation**: We will investigate and validate the vulnerability
3. **Communication**: We will keep you informed of our progress
4. **Credit**: With your permission, we will credit you in our security advisory

## Security Measures

OpenChat implements multiple layers of security:

### Authentication & Authorization

- **JWT Token Authentication**: Secure token-based authentication
- **OAuth 2.0 with PKCE**: Industry-standard secure authentication flow
- **SSO Integration**: Single sign-on via TitaniumVault
- **Role-Based Access Control**: Fine-grained permission system
- **Token Refresh**: Secure token refresh mechanism

### Data Protection

- **PostgreSQL Row Level Security (RLS)**: Database-level organization isolation
- **TLS Encryption**: All data encrypted in transit
- **Encrypted Storage**: Database encryption at rest
- **Input Validation**: Strict validation on all user inputs
- **SQL Injection Prevention**: Compile-time verified queries via SQLx

### Application Security

- **CSRF Protection**: Cross-site request forgery prevention
- **XSS Prevention**: Sanitized HTML output in markdown rendering
- **Rate Limiting**: Protection against brute force and DoS attacks
- **Secure Headers**: Security headers on all responses
- **CORS Configuration**: Restricted cross-origin access

### Infrastructure Security

- **Private VPC**: Database and services in private network
- **Network Load Balancer**: Secure traffic routing
- **CloudFront CDN**: DDoS protection for static assets
- **AWS ECS**: Containerized deployment with security groups

## Security Best Practices for Self-Hosting

If you're self-hosting OpenChat, follow these best practices:

### Environment Configuration

- Never commit `.env` files or credentials to version control
- Use strong, unique passwords for database and Redis
- Rotate secrets regularly
- Use environment variables for all sensitive configuration

### Network Security

- Deploy PostgreSQL and Redis in a private subnet
- Use TLS for all external connections
- Configure firewalls to restrict access
- Use a reverse proxy (nginx, Caddy) with TLS termination

### Database Security

- Enable PostgreSQL SSL connections
- Use strong database passwords
- Restrict database user permissions
- Enable Row Level Security (RLS) policies
- Regular backups with encryption

### Monitoring

- Enable audit logging
- Monitor for unusual activity
- Set up alerts for failed authentication attempts
- Review logs regularly

### Updates

- Keep OpenChat updated to the latest version
- Apply security patches promptly
- Monitor our GitHub releases for security advisories

## Disclosure Policy

- We follow responsible disclosure practices
- Security advisories will be published on GitHub after fixes are released
- We will coordinate disclosure timing with reporters
- Critical vulnerabilities may be disclosed sooner to protect users

## Security Updates

Security updates are announced via:

- GitHub Releases
- GitHub Security Advisories
- README changelog

## Contact

For security-related questions or concerns:

- **Email**: mackman42@outlook.com
- **GitHub**: [@mack42](https://github.com/mack42)

Thank you for helping keep OpenChat secure!
