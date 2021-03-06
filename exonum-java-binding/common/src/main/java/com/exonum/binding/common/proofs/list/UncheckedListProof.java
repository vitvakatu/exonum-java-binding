/*
 * Copyright 2018 The Exonum Team
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

package com.exonum.binding.common.proofs.list;

/**
 * A proof that some elements exist in a proof list. You must
 * {@link #check} its structure and root hash before accessing the elements.
 */
public interface UncheckedListProof {

  /**
   * Checks that a proof has either correct or incorrect structure and returns a CheckedListProof.
   */
  CheckedListProof check();

  /**
   * Returns raw source proof of this UncheckedListProof.
   */
  ListProofNode getRootProofNode();
}
