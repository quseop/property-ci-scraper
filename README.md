# Property CI Scraper

A comprehensive Rust web service that scrapes property data from the internet, stores it in a PostgreSQL database, and provides APIs for accessing the data. The collected data can be used to generate machine learning datasets for property intelligence.

## 🚀 Key Features

### 1. **Web Scraping**
- Configurable CSS selectors for different property websites
- Intelligent property container detection
- Price parsing with multiple currency formats
- Address parsing and geocoding support
- Duplicate detection via URL uniqueness
- Error handling and retry mechanisms

### 2. **Scheduled Jobs**
- Cron-based scheduling for periodic scraping
- Job management (create, update, delete, run manually)
- Real-time job monitoring and results
- Multiple predefined schedule templates
- Job status tracking and error reporting

### 3. **Data Storage**
- PostgreSQL database with optimized schema
- Property deduplication via unique constraints
- Indexed columns for fast querying
- Bulk insert operations for efficiency
- Database migrations support

### 4. **Advanced API**
- RESTful endpoints for all operations
- Property search with multiple filters
- Statistical analytics
- Real-time scraping metrics
- Health monitoring and status

### 5. **Machine Learning Dataset Export**
- CSV, JSON, and Parquet format support
- Feature engineering (price per sqm, categorical encoding)
- Configurable data filtering
- ML-ready datasets with engineered features
- Export statistics and data quality metrics

## 📋 Prerequisites

- Rust 1.70+ 
- PostgreSQL database
- Shuttle.rs account (for deployment)

## 🛠️ Installation

1. **Clone the repository**
```bash
git clone <repository-url>
cd property-ci-scraper
```

2. **Install dependencies**
```bash
cargo build
```

3. **Set up environment variables**
```bash
# For local development
export DATABASE_URL="postgresql://user:password@localhost/property_scraper"
```

4. **Run migrations** (handled automatically on startup)

## 🚀 Running the Service

### Local Development
```bash
cargo run
```

### Deploy to Shuttle.rs
```bash
cargo shuttle deploy
```

The service will be available at:
- Local: `http://localhost:8000`
- Deployed: Your Shuttle.rs URL

## 📚 API Documentation

### Base Endpoints

| Endpoint | Method | Description |
|----------|--------|-----------|
| `/` | GET | API information and documentation |
| `/health` | GET | Health check |

### Property Management

| Endpoint | Method | Description |
|----------|--------|-----------|
| `/properties` | GET | List all properties |
| `/properties/{id}` | GET | Get property by ID |
| `/properties` | POST | Create new property |
| `/properties/{id}` | PUT | Update property |
| `/api/v1/properties/search` | GET | Search properties with filters |
| `/api/v1/properties/recent` | GET | Get recent properties |
| `/api/v1/properties/stats` | GET | Property statistics |

### Scraping Job Management

| Endpoint | Method | Description |
|----------|--------|-----------|
| `/api/v1/scraping/jobs` | GET | List all scraping jobs |
| `/api/v1/scraping/jobs` | POST | Create new scraping job |
| `/api/v1/scraping/jobs/{id}` | GET | Get specific job |
| `/api/v1/scraping/jobs/{id}` | DELETE | Delete job |
| `/api/v1/scraping/jobs/{id}/run` | POST | Run job manually |
| `/api/v1/scraping/jobs/sample` | POST | Create sample job |
| `/api/v1/scraping/results` | GET | Get scraping results |
| `/api/v1/scraping/jobs/{id}/results` | GET | Get job-specific results |
| `/api/v1/scraping/stats` | GET | Scraping statistics |

### Data Export

| Endpoint | Method | Description |
|----------|--------|-----------|
| `/api/v1/export` | POST | Export properties data |
| `/api/v1/export/ml-dataset` | POST | Export ML-ready dataset |
| `/api/v1/export/stats` | GET | Export statistics |

## 🔧 Usage Examples

### 1. Create a Scraping Job

```json
POST /api/v1/scraping/jobs
{
  "name": "Example Property Site",
  "target_url": "https://example-realestate.com/listings",
  "selectors": {
    "title": "h2.listing-title",
    "price": "span.price-value",
    "address": "div.property-address",
    "property_type": "span.property-type",
    "bedrooms": "span.bedrooms",
    "bathrooms": "span.bathrooms",
    "land_size": "span.land-area",
    "floor_size": "span.floor-area"
  },
  "schedule": "0 0 2 * * *",
  "active": true
}
```

### 2. Search Properties

```bash
# Search by city
GET /api/v1/properties/search?city=Cape Town

# Search by price range
GET /api/v1/properties/search?min_price=500000&max_price=2000000

# Search by property type
GET /api/v1/properties/search?property_type=residential
```

### 3. Export ML Dataset

```json
POST /api/v1/export
{
  "format": "csv",
  "query": {
    "city": "Cape Town",
    "min_price": 100000
  },
  "include_metadata": true
}
```

### 4. Get Statistics

```bash
# Property statistics
GET /api/v1/properties/stats

# Scraping statistics  
GET /api/v1/scraping/stats

# Export statistics
GET /api/v1/export/stats
```

## 📊 Database Schema

The service uses a PostgreSQL database with the following main table:

```sql
CREATE TABLE properties (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    price BIGINT,
    address TEXT NOT NULL,
    province TEXT NOT NULL,
    city TEXT NOT NULL,
    suburb TEXT,
    property_type TEXT NOT NULL,
    bedrooms SMALLINT,
    bathrooms SMALLINT,
    garage_spaces SMALLINT,
    land_size DOUBLE PRECISION,
    floor_size DOUBLE PRECISION,
    source_url TEXT NOT NULL UNIQUE,
    latitude DOUBLE PRECISION,
    longitude DOUBLE PRECISION,
    scraped_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

## 🤖 Machine Learning Features

### Feature Engineering

The service automatically creates ML-ready features:

- **price_per_sqm_floor**: Price per square meter (floor area)
- **price_per_sqm_land**: Price per square meter (land area)  
- **property_type_encoded**: Hash-encoded property type
- **province_encoded**: Hash-encoded province
- **city_encoded**: Hash-encoded city
- **has_suburb**: Boolean indicator for suburb presence
- **price_category**: Categorized price ranges (low, medium, high, premium)

### Export Formats

- **CSV**: Most common for ML frameworks
- **JSON**: For web applications and APIs
- **Parquet**: For big data and analytics platforms

## ⚙️ Configuration

### Cron Schedule Examples

```rust
use crate::service::scheduler::CronSchedules;

// Predefined schedules
CronSchedules::DAILY        // "0 0 2 * * *" - 2 AM daily
CronSchedules::HOURLY       // "0 0 * * * *" - Every hour
CronSchedules::WEEKLY       // "0 0 2 * * 0" - 2 AM Sunday
CronSchedules::TWICE_DAILY  // "0 0 2,14 * * *" - 2 AM and 2 PM

// Custom schedules
CronSchedules::every_n_hours(6)      // Every 6 hours
CronSchedules::daily_at(9, 30)       // 9:30 AM daily
```

### CSS Selector Configuration

Configure selectors for different property websites:

```json
{
  "selectors": {
    "title": "h1.property-title, .listing-title",
    "price": ".price-display, .property-price span",
    "address": ".full-address, .property-location",
    "bedrooms": ".bedrooms .value, [data-testid='bedrooms']",
    "bathrooms": ".bathrooms .value, [data-testid='bathrooms']"
  }
}
```

## 🔍 Monitoring and Logging

The service provides comprehensive monitoring:

### Health Checks
```bash
GET /health
```

### Real-time Statistics
- Total properties scraped
- Properties scraped today
- Active scraping jobs
- Success/failure rates
- Average processing times

### Logging
- Structured logging with different levels
- Request/response logging
- Error tracking and reporting
- Performance metrics

## 🛡️ Error Handling

The service includes robust error handling:

- **Network errors**: Retry mechanisms with exponential backoff
- **Parsing errors**: Graceful degradation and error reporting
- **Database errors**: Transaction rollback and retry
- **Validation errors**: Clear error messages and status codes

## 🚀 Performance Optimizations

- **Bulk database operations**: Efficient batch inserts
- **Connection pooling**: Optimized database connections
- **Caching**: Smart caching of frequently accessed data
- **Indexing**: Database indexes for fast queries
- **Async processing**: Non-blocking I/O operations

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 📞 Support

For support and questions:
- Create an issue in the GitHub repository
- Check the API documentation at the root endpoint (`/`)
- Review the health status at `/health`

## 🔄 Version History

### v0.1.0 (Current)
- Initial release with core scraping functionality
- PostgreSQL integration
- RESTful API endpoints
- Scheduled job management
- ML dataset export capabilities
- Comprehensive monitoring and logging

---

**Built with ❤️ using Rust, Actix-Web, and PostgreSQL**

# Shuttle shared Postgres DB with Actix Web

This template shows how to connect a Postgres database and use it for a simple TODO list app.

## Example usage

```bash
curl -X POST -H 'content-type: application/json' localhost:8000/todos --data '{"note":"My todo"}'
# {"id":1,"note":"My todo"}

curl localhost:8000/todos/1
# {"id":1,"note":"My todo"}
```
