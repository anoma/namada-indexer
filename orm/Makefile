cp-test-migrations:
	rm -rf migrations_combined
	mkdir migrations_combined
	cp -r migrations/* migrations_combined
	cp -r migrations_for_tests/* migrations_combined

run-test-migrations:
	make cp-test-migrations
	diesel migration run --migration-dir migrations_combined

redo-test-migrations:
	make cp-test-migrations
	diesel migration redo --all --migration-dir migrations_combined
