#![allow(non_camel_case_types)]

use crate::stm32l4x3::{
    field_reader::FieldReader,
    generics::{Readable, RegisterSpec, Resettable, Writable, R as hR, W as hW},
};

#[doc = "Register `AR` reader"]
pub struct R(hR<AR_SPEC>);
impl core::ops::Deref for R {
    type Target = hR<AR_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<hR<AR_SPEC>> for R {
    #[inline(always)]
    fn from(reader: hR<AR_SPEC>) -> Self {
        R(reader)
    }
}
#[doc = "Register `AR` writer"]
pub struct W(hW<AR_SPEC>);
impl core::ops::Deref for W {
    type Target = hW<AR_SPEC>;
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
impl From<hW<AR_SPEC>> for W {
    #[inline(always)]
    fn from(writer: hW<AR_SPEC>) -> Self {
        W(writer)
    }
}
#[doc = "Field `ADDRESS` reader - Address"]
pub struct ADDRESS_R(FieldReader<u32, u32>);
impl ADDRESS_R {
    #[inline(always)]
    pub(crate) fn new(bits: u32) -> Self {
        ADDRESS_R(FieldReader::new(bits))
    }
}
impl core::ops::Deref for ADDRESS_R {
    type Target = FieldReader<u32, u32>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[doc = "Field `ADDRESS` writer - Address"]
pub struct ADDRESS_W<'a> {
    w: &'a mut W,
}
impl<'a> ADDRESS_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u32) -> &'a mut W {
        self.w.bits = value;
        self.w
    }
}
impl R {
    #[doc = "Bits 0:31 - Address"]
    #[inline(always)]
    pub fn address(&self) -> ADDRESS_R {
        ADDRESS_R::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Address"]
    #[inline(always)]
    pub fn address(&mut self) -> ADDRESS_W {
        ADDRESS_W { w: self }
    }
    #[doc = "Writes raw bits to the register."]
    #[inline(always)]
    pub unsafe fn bits(&mut self, bits: u32) -> &mut Self {
        self.0.bits(bits);
        self
    }
}
#[doc = "address register\n\nThis register you can [`read`](crate::generic::Reg::read), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ar](index.html) module"]
pub struct AR_SPEC;
impl RegisterSpec for AR_SPEC {
    type Ux = u32;
}
#[doc = "`read()` method returns [ar::R](R) reader structure"]
impl Readable for AR_SPEC {
    type Reader = R;
}
#[doc = "`write(|w| ..)` method takes [ar::W](W) writer structure"]
impl Writable for AR_SPEC {
    type Writer = W;
}
#[doc = "`reset()` method sets AR to value 0"]
impl Resettable for AR_SPEC {
    #[inline(always)]
    fn reset_value() -> Self::Ux {
        0
    }
}
