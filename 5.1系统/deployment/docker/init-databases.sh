#!/bin/bash
set -e

# 创建所有必需的数据库
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    CREATE DATABASE logging_db;
    CREATE DATABASE cleaning_db;
    CREATE DATABASE strategy_db;
    CREATE DATABASE performance_db;
    CREATE DATABASE trading_db;
    CREATE DATABASE ai_model_db;
    CREATE DATABASE config_db;
    
    -- 为每个数据库创建专用用户
    CREATE USER logging_user WITH PASSWORD 'logging_pass';
    CREATE USER cleaning_user WITH PASSWORD 'cleaning_pass';
    CREATE USER strategy_user WITH PASSWORD 'strategy_pass';
    CREATE USER performance_user WITH PASSWORD 'performance_pass';
    CREATE USER trading_user WITH PASSWORD 'trading_pass';
    CREATE USER ai_model_user WITH PASSWORD 'ai_model_pass';
    CREATE USER config_user WITH PASSWORD 'config_pass';
    
    -- 授予权限
    GRANT ALL PRIVILEGES ON DATABASE logging_db TO logging_user;
    GRANT ALL PRIVILEGES ON DATABASE cleaning_db TO cleaning_user;
    GRANT ALL PRIVILEGES ON DATABASE strategy_db TO strategy_user;
    GRANT ALL PRIVILEGES ON DATABASE performance_db TO performance_user;
    GRANT ALL PRIVILEGES ON DATABASE trading_db TO trading_user;
    GRANT ALL PRIVILEGES ON DATABASE ai_model_db TO ai_model_user;
    GRANT ALL PRIVILEGES ON DATABASE config_db TO config_user;
EOSQL

echo "All databases created successfully!"