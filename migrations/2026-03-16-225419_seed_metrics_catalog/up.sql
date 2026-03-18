-- Update category CHECK constraint to match new naming
ALTER TABLE metrics_catalog DROP CONSTRAINT metrics_catalog_category_check;
ALTER TABLE metrics_catalog ADD CONSTRAINT metrics_catalog_category_check CHECK (
    category IN ('valuation', 'profitability', 'growth', 'financial_health', 'investor_return')
);

-- Valuation (lower is better for all)
INSERT INTO metrics_catalog (key, name, category, data_type, unit, higher_is_better) VALUES
    ('pettm',      'P/E Ratio (TTM)',   'valuation', 'multiple', 'x', FALSE),
    ('peg',        'PEG Ratio',         'valuation', 'multiple', 'x', FALSE),
    ('ev2ebitda',  'EV/EBITDA',         'valuation', 'multiple', 'x', FALSE),
    ('ps',         'Price/Sales',       'valuation', 'multiple', 'x', FALSE),
    ('pb',         'Price/Book',        'valuation', 'multiple', 'x', FALSE),
    ('pfcf',       'Price/FCF',         'valuation', 'multiple', 'x', FALSE);

-- Growth (higher is better for all)
INSERT INTO metrics_catalog (key, name, category, data_type, unit, higher_is_better) VALUES
    ('rvn_growth_3y',      '3Y Revenue Growth',    'growth', 'percent', '%', TRUE),
    ('ebitda_growth_3y',   '3Y EBITDA Growth',     'growth', 'percent', '%', TRUE),
    ('cashflow_growth_3y', '3Y FCF Growth',         'growth', 'percent', '%', TRUE),
    ('earning_growth_3y',  '3Y EPS Growth',         'growth', 'percent', '%', TRUE),
    ('book_growth_3y',     '3Y Book Value Growth',  'growth', 'percent', '%', TRUE);

-- Profitability (higher is better for all)
INSERT INTO metrics_catalog (key, name, category, data_type, unit, higher_is_better) VALUES
    ('roic',         'ROIC',              'profitability', 'percent', '%', TRUE),
    ('oprt_margain', 'Operating Margin',  'profitability', 'percent', '%', TRUE),
    ('FCFmargin',    'FCF Margin',        'profitability', 'percent', '%', TRUE),
    ('roa',          'ROA',               'profitability', 'percent', '%', TRUE),
    ('net_margain',  'Net Margin',        'profitability', 'percent', '%', TRUE);

-- Financial Health
INSERT INTO metrics_catalog (key, name, category, data_type, unit, higher_is_better) VALUES
    ('interest_coverage', 'Interest Coverage',  'financial_health', 'multiple', 'x', TRUE),
    ('debt2ebitda',       'Net Debt/EBITDA',    'financial_health', 'multiple', 'x', FALSE),
    ('quick_ratio',       'Quick Ratio',        'financial_health', 'multiple', 'x', TRUE),
    ('cash2debt',         'Cash/Debt',          'financial_health', 'multiple', 'x', TRUE),
    ('equity2asset',      'Equity/Assets',      'financial_health', 'percent',  '%', TRUE);

-- Investor Return
INSERT INTO metrics_catalog (key, name, category, data_type, unit, higher_is_better) VALUES
    ('ForwardDividendYield', 'Dividend Yield',      'investor_return', 'percent', '%', TRUE),
    ('buyback_yield',        'Buyback Yield',       'investor_return', 'percent', '%', TRUE),
    ('dividend_growth_3y',   'Dividend Growth 3Y',  'investor_return', 'percent', '%', TRUE),
    ('payout',               'Payout Ratio',        'investor_return', 'percent', '%', FALSE),
    ('shareholder_yield',    'Shareholder Yield',   'investor_return', 'percent', '%', TRUE);
