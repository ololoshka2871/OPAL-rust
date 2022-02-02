#![allow(non_camel_case_types)]

use crate::stm32l4x3::{
    field_reader::FieldReader,
    generics::{Readable, RegisterSpec, Resettable, Writable, R as hR, W as hW},
};

#[doc = "Register `PSMKR` reader"]
pub struct R(hR<PSMKR_SPEC>);
impl core::ops::Deref for R {
    type Target = hR<PSMKR_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<hR<PSMKR_SPEC>> for R {
    #[inline(always)]
    fn from(reader: hR<PSMKR_SPEC>) -> Self {
        R(reader)
    }
}
#[doc = "Register `PSMKR` writer"]
pub struct W(hW<PSMKR_SPEC>);
impl core::ops::Deref for W {
    type Target = hW<PSMKR_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl core::ops::DerefMut for W {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl From<hW<PSMKR_SPEC>> for W {
    #[inline(always)]
    fn from(writer: hW<PSMKR_SPEC>) -> Self {
        W(writer)
    }
}
#[doc = "Field `MASK` reader - Status mask"]
pub struct MASK_R(FieldReader<u32, u32>);
impl MASK_R {
    #[inline(always)]
    pub(crate) fn new(bits: u32) -> Self {
        MASK_R(FieldReader::new(bits))
    }
}
impl core::ops::Deref for MASK_R {
    type Target = FieldReader<u32, u32>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[doc = "Field `MASK` writer - Status mask"]
pub struct MASK_W<'a> {
    w: &'a mut W,
}
impl<'a> MASK_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u32) -> &'a mut W {
        self.w.bits = value;
        self.w
    }
}
impl R {
    #[doc = "Bits 0:31 - Status mask"]
    #[inline(always)]
    pub fn mask(&self) -> MASK_R {
        MASK_R::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Status mask"]
    #[inline(always)]
    pub fn mask(&mut self) -> MASK_W {
        MASK_W { w: self }
    }
    #[doc = "Writes raw bits to the register."]
    #[inline(always)]
    pub unsafe fn bits(&mut self, bits: u32) -> &mut Self {
        self.0.bits(bits);
        self
    }
}
#[doc = "polling status mask register\n\nThis register you can [`read`](crate::generic::Reg::read), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [psmkr](index.html) module"]
pub struct PSMKR_SPEC;
impl RegisterSpec for PSMKR_SPEC {
    type Ux = u32;
}
#[doc = "`read()` method returns [psmkr::R](R) reader structure"]
impl Readable for PSMKR_SPEC {
    type Reader = R;
}
#[doc = "`write(|w| ..)` method takes [psmkr::W](W) writer structure"]
impl Writable for PSMKR_SPEC {
    type Writer = W;
}
#[doc = "`reset()` method sets PSMKR to value 0"]
impl Resettable for PSMKR_SPEC {
    #[inline(always)]
    fn reset_value() -> Self::Ux {
        0
    }
}
