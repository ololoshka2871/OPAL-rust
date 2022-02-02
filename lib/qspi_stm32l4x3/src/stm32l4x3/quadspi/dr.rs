#![allow(non_camel_case_types)]

use crate::stm32l4x3::{
    field_reader::FieldReader,
    generics::{Readable, RegisterSpec, Resettable, Writable, R as hR, W as hW},
};

#[doc = "Register `DR` reader"]
pub struct R(hR<DR_SPEC>);
impl core::ops::Deref for R {
    type Target = hR<DR_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<hR<DR_SPEC>> for R {
    #[inline(always)]
    fn from(reader: hR<DR_SPEC>) -> Self {
        R(reader)
    }
}
#[doc = "Register `DR` writer"]
pub struct W(hW<DR_SPEC>);
impl core::ops::Deref for W {
    type Target = hW<DR_SPEC>;
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
impl From<hW<DR_SPEC>> for W {
    #[inline(always)]
    fn from(writer: hW<DR_SPEC>) -> Self {
        W(writer)
    }
}
#[doc = "Field `DATA` reader - Data"]
pub struct DATA_R(FieldReader<u32, u32>);
impl DATA_R {
    #[inline(always)]
    pub(crate) fn new(bits: u32) -> Self {
        DATA_R(FieldReader::new(bits))
    }
}
impl core::ops::Deref for DATA_R {
    type Target = FieldReader<u32, u32>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[doc = "Field `DATA` writer - Data"]
pub struct DATA_W<'a> {
    w: &'a mut W,
}
impl<'a> DATA_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u32) -> &'a mut W {
        self.w.bits = value;
        self.w
    }
}
impl R {
    #[doc = "Bits 0:31 - Data"]
    #[inline(always)]
    pub fn data(&self) -> DATA_R {
        DATA_R::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Data"]
    #[inline(always)]
    pub fn data(&mut self) -> DATA_W {
        DATA_W { w: self }
    }
    #[doc = "Writes raw bits to the register."]
    #[inline(always)]
    pub unsafe fn bits(&mut self, bits: u32) -> &mut Self {
        self.0.bits(bits);
        self
    }
}
#[doc = "data register\n\nThis register you can [`read`](crate::generic::Reg::read), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [dr](index.html) module"]
pub struct DR_SPEC;
impl RegisterSpec for DR_SPEC {
    type Ux = u32;
}
#[doc = "`read()` method returns [dr::R](R) reader structure"]
impl Readable for DR_SPEC {
    type Reader = R;
}
#[doc = "`write(|w| ..)` method takes [dr::W](W) writer structure"]
impl Writable for DR_SPEC {
    type Writer = W;
}
#[doc = "`reset()` method sets DR to value 0"]
impl Resettable for DR_SPEC {
    #[inline(always)]
    fn reset_value() -> Self::Ux {
        0
    }
}
