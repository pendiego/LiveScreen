package io.github.pendiego.livescreen

import org.junit.Assert.assertEquals
import org.junit.Test

class AvatarTest {
    @Test
    fun autoKeyMatchesContractVectors() {
        // Vetores oficiais acordados com o núcleo (livescreen_core::avatar::auto)
        assertEquals(3, AvatarData.auto("Diego"))
        assertEquals(6, AvatarData.auto("Alice"))
        assertEquals(7, AvatarData.auto("#0"))

        // Casos adicionais de hashing FNV-1a 32-bit sobre UTF-8
        assertEquals(1, AvatarData.auto("Host"))
        assertEquals(3, AvatarData.auto(""))
        assertEquals(5, AvatarData.auto("LiveScreen"))
    }

    @Test
    fun resolveUsesChosenWhenValidOtherwiseAuto() {
        // 0 = automático (calcula pela chave)
        assertEquals(3, AvatarData.resolve(0, "Diego"))
        assertEquals(6, AvatarData.resolve(0, "Alice"))
        assertEquals(7, AvatarData.resolve(0, "#0"))

        // Escolha explícita válida (1..7) sempre preservada
        assertEquals(1, AvatarData.resolve(1, "Diego"))
        assertEquals(2, AvatarData.resolve(2, "Alice"))
        assertEquals(7, AvatarData.resolve(7, "Diego"))

        // Fora da faixa cai no automático
        assertEquals(3, AvatarData.resolve(8, "Diego"))
        assertEquals(6, AvatarData.resolve(99, "Alice"))
        assertEquals(7, AvatarData.resolve(-1, "#0"))
    }
}
