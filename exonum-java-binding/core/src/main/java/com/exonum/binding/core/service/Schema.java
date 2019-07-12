/*
 * Copyright 2019 The Exonum Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package com.exonum.binding.core.service;

import com.exonum.binding.common.hash.HashCode;
import java.util.List;

/**
 * A schema of the collections (a.k.a. indices) of a service.
 */
public interface Schema {

  /**
   * Returns the root hashes of Merkelized tables in this database schema, as of the current
   * state of the database. If there are no Merkelized tables, returns an empty list.
   *
   * <p>This list of root hashes represents the current service state. Lists of these hashes
   * from each service are aggregated in a single <em>blockchain state hash</em> that reflects
   * the state of all services in the blockchain.
   */
  List<HashCode> getStateHashes();
}