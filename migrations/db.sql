-- elo_tracker.summoners definition

CREATE TABLE `summoners` (
  `discord_id` bigint(20) NOT NULL,
  `summoner_id` text COLLATE utf8mb4_unicode_ci NOT NULL,
  `rank` varchar(100) COLLATE utf8mb4_unicode_ci NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;