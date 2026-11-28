ALTER TABLE rawrarr_eth_auth_challenges
    DROP CONSTRAINT rawrarr_eth_auth_challenges_flow_check,
    DROP CONSTRAINT rawrarr_eth_auth_challenges_check,
    ADD CONSTRAINT rawrarr_eth_auth_challenges_flow_check
        CHECK (flow IN ('wallet_login', 'wallet_signup', 'wallet_link')),
    ADD CONSTRAINT rawrarr_eth_auth_challenges_check
        CHECK ((flow IN ('wallet_login', 'wallet_link')) = (user_id IS NOT NULL));
